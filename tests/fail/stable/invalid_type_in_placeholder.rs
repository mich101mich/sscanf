fn main() {
    sscanf::sscanf!("hi", "{.}");
    sscanf::sscanf!("hi", "{bob}");
    sscanf::sscanf!("hi", "{.:/hi/}");
    sscanf::sscanf!("hi", "{bob:/hi/}");
    sscanf::sscanf!("hi", "{99}");
}
