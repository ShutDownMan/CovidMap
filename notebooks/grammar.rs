peg::parser!{
  grammar list_parser() for str {
    /// One or more operators makes up a query

    rule root() -> str
        = __ * query() __ *;

    rule query() -> str
        = (operator() __ *) +;
    
    /// There are realy only two kinds of operator: include and exclude

    rule operator() =>
        = excluding() / including();

    ///  To exclude, you use a "-" before the query. OR cannot be negated unless it's in a group.

    rule excluding() =>
        = "-" base();
    
    rule including() =>
        = "" (or() / base());
    
    /// Operators can be grouped with an OR keyword in groups of two or more.

    rule or() =>
        = orable() or_groups();

    rule or_groups() =>
        = (or_sep() orable())+;

    rule orable() =>
        = "" base();

    /// The base operators are the things that make up the query itself.

    rule base() =>
        = group()
        / list()
        / url()
        / pair()
        / exact()
        / word();

    /// A group is a complete subquery. Recursion!

    rule group() =>
        = "(" root ")";

    /// Query Syntax Rules
    /// These outline the specifics of the syntax

    rule url() =>
        = slug url_sep v:word;

    rule pair() =>
        = slug sep v:word;

    rule exact() =>
        = '"' ([^ '"']*) '"';

    rule word() =>
        = "+"? [^ __ ')(']+;

    rule or_sep() =>
        = __+ "OR" __+;

    rule date() =>
        = d() d() d() d() "-" d() d() "-" d() d()+;

    rule slug() =>
        = (['a'...='z'] | ['A'...='Z'] | ['0'...='9'] | '_')+;

    rule url_sep() =>
        = "://";

    rule sep() =>
        = ":";

    rule integer() =>
        = d() d()*;

    rule d() =>
        = ['0'..='9'];

}

pub fn main() {
    assert_eq!(list_parser::list("[1,1,2,3,5,8]"), Ok(vec![1, 1, 2, 3, 5, 8]));
}