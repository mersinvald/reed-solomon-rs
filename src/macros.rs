macro_rules! polynom {
    [$value:expr; $count:expr] => {{
        let array = [$value; $count];
        $crate::gf::poly::Polynom::from(&array[..])
    }}; 

    [$( $value:expr ),* ] => {{
        let array = [$($value, )*];
        $crate::gf::poly::Polynom::from(&array[..])
    }};
}

macro_rules! uncheck {
    ($array:ident[$index:expr]) => {
        if cfg!(feature = "unsafe_indexing") {
            unsafe {
                *$array.get_unchecked($index)
            }
        } else {
            $array[$index]
        }
    }
}

macro_rules! uncheck_mut {
    ($array:ident[$index:expr]) => {
        * if cfg!(feature = "unsafe_indexing") {
            unsafe {
                $array.get_unchecked_mut($index)
            }
        } else {
            &mut $array[$index]
        }
    }
}
