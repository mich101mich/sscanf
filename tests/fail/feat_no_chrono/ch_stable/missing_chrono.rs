fn main() {
    let input = "2021-06-21T13:37:42+04:30";
    let parsed = sscanf::scanf!(input, "{:%Y-%m-%dT%H:%M:%S%:z}", DateTime);
    let parsed = sscanf::scanf!(input, "{DateTime:%Y-%m-%dT%H:%M:%S%:z}");
}
