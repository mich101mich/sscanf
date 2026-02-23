mod module {
    pub struct NoFromScanf;
}

fn main() {
    ///// Type passed as argument does not implement FromScanf /////
    sscanf::sscanf!("hi", "{}", module::NoFromScanf);
    //~                         ^^^^^^^^^^^^^^^^^^^ error: type `NoFromScanf` can't be parsed by sscanf because it does not implement `FromScanf`
    //~                                             label: can't be parsed by sscanf

    ///// Type inside placeholder does not implement FromScanf /////
    sscanf::sscanf!("hi", "{module::NoFromScanf}");
    //~                   ^^^^^^^^^^^^^^^^^^^^^^^ error: type `NoFromScanf` can't be parsed by sscanf because it does not implement `FromScanf`
    //~                                           label: can't be parsed by sscanf

    ///// Imported type does not implement FromScanf /////
    use module::NoFromScanf;
    sscanf::sscanf!("hi", "{}", NoFromScanf);
    //~                         ^^^^^^^^^^^ error: type `NoFromScanf` can't be parsed by sscanf because it does not implement `FromScanf`
    //~                                     label: can't be parsed by sscanf

    ///// Imported type used in placeholder /////
    use module::NoFromScanf;
    sscanf::sscanf!("hi", "{NoFromScanf}");
    //~                   ^^^^^^^^^^^^^^^ error: type `NoFromScanf` can't be parsed by sscanf because it does not implement `FromScanf`
    //~                                   label: can't be parsed by sscanf

    ////////////////////////////////////////////////////////////////////////////////
}
