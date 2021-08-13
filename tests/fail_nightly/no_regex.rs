struct Test;
impl std::str::FromStr for Test {
    type Err = ();
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Test)
    }
}

fn main() {
    sscanf::scanf!("hi", "{}", Test);
}
