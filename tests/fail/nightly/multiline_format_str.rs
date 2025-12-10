fn main() {
    sscanf::sscanf!(
        input,
        "This
is a fau{ty
multiline string!"
    )
    .unwrap();

    sscanf::sscanf!(
        input,
        "{This
is another faulty
multiline string!"
    )
    .unwrap();

    sscanf::sscanf!(
        input,
        "And so
is
this}"
    )
    .unwrap();

    sscanf::sscanf!(
        input,
        "This
is a 😟 fau{ty
multiline string!"
    )
    .unwrap();

    sscanf::sscanf!(input, "This\nis a\n{fake\nmultiline string!").unwrap();
}
