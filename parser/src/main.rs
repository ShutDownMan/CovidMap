use json::JsonValue;
use json::object;
use json::array;
use regex::Regex;

peg::parser!{
	grammar query() for str {

		/// One or more operators makes up a query
		pub rule parse() -> json::JsonValue
			=  p:(operator())+ { object!{ "query": p } }

		/// There are realy only two kinds of operator: include and exclude
		rule operator() -> json::JsonValue
			= v:(excluding() / including()) { v };

		///  To exclude, you use a "-" before the query. OR cannot be negated unless it's in a group.
		rule excluding() -> json::JsonValue
			= "-" v:base() { object!{ "excluding": v } };

		rule including() -> json::JsonValue
			= "" v:(or() / base()) { object!{ "including": v } };

		/// Operators can be grouped with an OR keyword in groups of two or more.
		rule or() -> json::JsonValue
			= v:$(orable() or_groups()) {
				let words = Regex::new(r"\s+OR\s+")
					.unwrap()
					.split(v)
					.collect::<Vec<&str>>();

				object!{ "or": JsonValue::from(words)}
			};

		rule or_groups() -> json::JsonValue
			= v:$(or_sep() orable())+ { object!{ "or_groups": v }};

		rule orable() -> json::JsonValue
			= "" v:base() { object!{ "orable": v }};

		rule or_sep() = ___() "OR" ___();

		rule base() -> json::JsonValue
			= v:(group() / pair() / url() / exact() / atom()) { v };

		/// FIX key value pair
		rule url() -> json::JsonValue
			= k:slug() url_sep() v:word() { object!{ k: v } };

		rule pair() -> json::JsonValue
			= k:slug() pair_sep() v:word() { object!{ k: v } };

		rule exact() -> json::JsonValue
			= "\"" v:$([^ '"']*) "\"" { object!{ "exact": v } };

		/// A group is a complete subquery. Recursion!
		rule group() -> json::JsonValue
			= ___() "(" v:parse() ")" ___() { object!{ "group": v } };

		rule ___() = [' ' | '\n' | '\t']*
		
		rule word() -> String
			= ___() w:$([^'-'] ['a'..='z' | 'A'..='Z' | '-']+) ___() {
				w.trim().to_string()
			}

		rule atom() -> json::JsonValue
			= v:word() { JsonValue::from(v) }

		rule pair_sep() = ":";

		rule url_sep() = "://";

		rule slug() -> String
			= ___() v:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-']+) ___() { v.trim().to_string() };

		// /// One or more operators makes up a query
		// rule root() -> str
		// 	= ___() query() ___();

		// rule query() -> str
		// 	= (operator() ___()) +;

		// /// There are realy only two kinds of operator: include and exclude
		// rule operator() -> str
		// 	= excluding() / including();

		// ///  To exclude, you use a "-" before the query. OR cannot be negated unless it's in a group.

		// rule excluding() -> str
		// 	= "-" base();

		// rule including() -> str
		// 	= "" (or() / base());

		// /// Operators can be grouped with an OR keyword in groups of two or more.
		// rule or() -> str
		// 	= orable() or_groups();

		// rule or_groups() -> str
		// 	= (or_sep() orable())+;

		// rule orable() -> str
		// 	= "" base();

		// /// The base operators are the things that make up the query itself.
		// rule base() -> str
		// 	= group()
		// 	/ url()
		// 	/ pair()
		// 	/ exact()
		// 	/ word();

		// /// A group is a complete subquery. Recursion!
		// rule group() -> str
		// 	= "(" root() ")";

		// /// Query Syntax Rules
		// /// These outline the specifics of the syntax
		// rule url() -> str
		// 	= slug() url_sep() v:word();

		// rule pair() -> str
		// 	= slug() sep() v:word();

		// rule exact() -> str
		// 	= '"' ([^ '"']*) '"';

		// rule word() -> str
		// 	= "+"? ([^ ___()] / [^ ")("])+;

		// rule or_sep() -> str
		// 	= ___()+ "OR" ___()+;

		// rule date() -> str
		// 	= d() d() d() d() "-" d() d() "-" d() d()+;

		// rule slug() -> str
		// 	= (['a'...='z'] / ['A'...='Z'] / ['0'...='9'] / '_')+;

		// rule url_sep() -> str
		// 	= "://";

		// rule sep() -> str
		// 	= ":";

		// rule integer() -> str
		// 	= d() d()*;

		// rule d() -> str
		// 	= ['0'..='9'];
	}
}

pub fn main() {
	let ast = query::parse(" from:place ");
	println!("{:#}", ast.unwrap());
	// assert_eq!(list_parser::list("[1,1,2,3,5,8]"), Ok(vec![1, 1, 2, 3, 5, 8]));
}