fn main() {
    sscanf::sscanf!(
        "Hi",
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
        "Hi",
        "This
is another fau{ty
multiline string!"
    )
    .unwrap();

    sscanf::sscanf!(
        "Hi",
        "And so
is
this}, but
the error is only
on one line"
    )
    .unwrap();

    sscanf::sscanf!(
        "Hi",
        "This
is a 😟 fau{ty
multiline string with unicode!"
    )
    .unwrap();

    sscanf::sscanf!("Hi", "This\nis a\n{fake\nmultiline string!").unwrap();
}
