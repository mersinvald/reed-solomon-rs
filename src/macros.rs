macro_rules! polynom {
    [$value:expr; $count:expr] => {
        $crate::gf::poly::Polynom::copy_from_slice(&[$value; $count])
    }; 

    [$( $value:expr ),* ] => {
        $crate::gf::poly::Polynom::copy_from_slice(&[$($value, )*])
    };
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