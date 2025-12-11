fn main() {
    sscanf::sscanf!(
        input,
        "{This
is
a
faulty
multiline
string
that
is
really
long!"
    )
    .unwrap();

    sscanf::sscanf!(
        input,
        "This
is another fau{ty
multiline string!"
    )
    .unwrap();

    sscanf::sscanf!(
        input,
        "And so
is
this}, but
the error is only
on one line"
    )
    .unwrap();

    sscanf::sscanf!(
        input,
        "This
is a 😟 fau{ty
multiline string with unicode!"
    )
    .unwrap();

    sscanf::sscanf!(input, "This\nis a\n{fake\nmultiline string!").unwrap();
}
