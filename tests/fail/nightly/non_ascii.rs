fn main() {
    type Ay̆y̆y̆ = u8;
    sscanf::sscanf!("Hi", "y̆😛y̆{Ay̆y̆y̆:😛}y̆😛y̆");
    sscanf::sscanf!("Hi", r##"y̆👨‍👩‍👧‍👦y̆{Ay̆y̆y̆:😛}y̆😛y̆"##);
    sscanf::sscanf!("Hi", "y\u{306}\u{1f61b}😛y\u{306}{Ay\u{306}y\u{306}y\u{306}:\u{1f61b}}y\u{306}\u{1f61b}y\u{306}");
}