peg::parser!{
	grammar query() for str {

		pub rule parse() -> Vec<String>
			=  p:(word())+  { p }

		rule word() -> String
			= ___() w:$(['a'..='z' | 'A'..='Z']+) ___() { w.to_string() }

		rule ___() = [' ' | '\n' | '\t']*
		
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
		// 	/ list()
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
	println!("{:?}", query::parse(" test de palavra "));
	// assert_eq!(list_parser::list("[1,1,2,3,5,8]"), Ok(vec![1, 1, 2, 3, 5, 8]));
}