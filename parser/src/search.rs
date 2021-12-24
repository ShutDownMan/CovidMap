use json::JsonValue;
use json::object;

peg::parser!{
    pub grammar query() for str {

        /// One or more operators makes up a query
        pub rule parse() -> json::JsonValue
            = ___() p:(operator())+ ___() { object!{ "query": p } }

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
            = h_v:orable() t_v:or_groups()+ {
                let mut or_list = t_v.clone();
                or_list.insert(0, h_v);
                
                object!{ "or": JsonValue::from(or_list)}
            };

        rule or_groups() -> json::JsonValue
            = or_sep() v:orable() { JsonValue::from(v) };

        rule orable() -> json::JsonValue
            = v:base() { v };

        rule or_sep() = ___() "OR" ___();

        rule base() -> json::JsonValue
            = v:(group() / pair() / url() / exact() / atom()) { v };

        /// FIX key value pair
        rule url() -> json::JsonValue
            = k:word() url_sep() v:word() { object!{ "pair": { "key": k, "value": v } } };

        rule pair() -> json::JsonValue
            = k:word() pair_sep() v:word() { object!{ "pair": { "key": k, "value": v } } };

        rule exact() -> json::JsonValue
            = r#"""# v:$([^ '"']*) r#"""# { object!{ "exact": v } };

        /// A group is a complete subquery. Recursion!
        rule group() -> json::JsonValue
            = ___() "(" v:parse() ")" ___() { object!{ "group": v } };

        rule atom() -> json::JsonValue
            = v:word() { JsonValue::from(v) }

        rule word() -> String
            = ___() w:$([^'-'] (char_regular() / ['-' | '_'])+) ___() {
                w.trim().to_string()
            }

        rule char_regular() -> String
            = v:$(quiet!{[_]} / expected!("regular character")) {?
                let c = v.chars().nth(0).unwrap();
                if c.is_alphanumeric() { Ok(c.to_string()) } else { Err("") }
            }

        rule pair_sep() = ":";

        rule url_sep() = "://";
    
        rule _() = quiet!{" "} / expected!("{SP}")
        rule __() = quiet!{_ / "\t"} / expected!("{WS}")
        rule ___() = quiet!{[' ' | '\n' | '\t']*} / expected!("{MWS}")
    }
}

pub fn ast_to_query(ast: &json::JsonValue) -> String {
    let root = &ast["query"];

    // println!("{:#}", root.len());

    root.members()
        .filter_map(|x| { get_query_from_base(x) })
        .collect::<Vec<String>>()
        .join(" && ")
}

pub fn get_query_from_base(base: &json::JsonValue) -> Option<String> {
    match base {
        _ if base.is_string() => {
            Some(format!("to_tsquery('{}')", base.to_string()))
        }
        _ if base.has_key("including") => {
            get_query_from_base(&base["including"])
        },
        _ if base.has_key("excluding") => {
            Some(format!("!!( {} )", get_query_from_base(&base["excluding"])?))
        },
        _ if base.has_key("or") => {
            Some(base["or"].members()
                .filter_map(|x| { get_query_from_base(x) })
                .collect::<Vec<String>>()
                .join(" || "))
        },
        _ if base.has_key("group") => {
            Some(format!("( {} )", ast_to_query(&base["group"]) ))
        },
        _ if base.has_key("exact") => {
            Some(format!("'{}'::tsquery", base["exact"].to_string()))
        },
        _ => { None }
    }
}