fn main() {
    ///// Invalid regex flag /////
    sscanf::sscanf!("", "{://g}", str);
    //~                      ^ unknown format option starting with 'g'.
    //~                        Use 'b', 'o', 'x' or 'r' for number format options, '/' for regex, or '[' for custom format options

    ///// Empty regex /////
    sscanf::sscanf!("", "{:/}", str);
    //~                    ^^ missing '/' to close the regex option

    ///// Unclosed regex /////
    sscanf::sscanf!("", "{:/", str);
    //~                    ^ missing '/' to close the regex option

    ///// Unclosed regex with escaped delimiter /////
    sscanf::sscanf!("", r"{:/\", str);
    //~                     ^^ missing '/' to close the regex option

    ///// Unclosed regex with escaped backslash /////
    sscanf::sscanf!("", "{:/\\", str);
    //~                    ^^^ missing '/' to close the regex option

    ///// Unclosed regex with escaped slash /////
    sscanf::sscanf!("", r"{:/\/", str);
    //~                     ^^^ missing unescaped '/' to close the regex option

    ///// Unclosed regex with escaped slash (standard string) /////
    sscanf::sscanf!("", "{:/\\/", str);
    //~                    ^^^^ missing unescaped '/' to close the regex option

    ///// Invalid regex (unclosed character class) /////
    sscanf::sscanf!("", r"{:/abc[def/}", str);
    //~                     ^^^^^^^^^ invalid regex override: regex parse error:
    //~                                   abc[def
    //~                                      ^
    //~                               error: unclosed character class

    ///// Invalid regex (repetition operator missing expression) /////
    sscanf::sscanf!("", r"{:/{/}", str);
    //~                     ^^^ invalid regex override: regex parse error:
    //~                             {
    //~                             ^
    //~                         error: repetition operator missing expression

    ///// Invalid regex (escaped brace repetition operator) /////
    sscanf::sscanf!("", r"{:/{\}/}", str);
    //~                     ^^^^^ invalid regex override: regex parse error:
    //~                               {\}
    //~                               ^
    //~                           error: repetition operator missing expression

    ////////////////////////////////////////////////////////////////////////////////
}
