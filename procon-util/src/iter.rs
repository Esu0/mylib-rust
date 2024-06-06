pub trait Tuple<const N: usize> {
    type Item;
    fn into_val(self) -> Self::Item;
}

// impl<T1, T2> Tuple<0> for (T1, T2) {
//     type Item = T1;
//     fn into_val(self) -> Self::Item {
//         self.0
//     }
// }

// impl<'a, T1, T2> Tuple<0> for &'a (T1, T2) {
//     type Item = &'a T1;
//     fn into_val(self) -> Self::Item {
//         &self.0
//     }
// }

macro_rules! tuple_impl {
    ($ind:expr, $item:ty, $ret:ident, [ $($t:ident),+ ], [$($name:ident),+]) => {
        impl<$($t),*> Tuple<$ind> for ($($t),*) {
            type Item = $item;
            fn into_val(self) -> Self::Item {
                let ($($name),*) = self;
                $ret
            }
        }

        impl<'a, $($t),*> Tuple<$ind> for &'a ($($t),*) {
            type Item = &'a $item;
            fn into_val(self) -> Self::Item {
                let ($($name),*) = self;
                $ret
            }
        }

        impl<'a, $($t),*> Tuple<$ind> for &'a mut ($($t),*) {
            type Item = &'a mut $item;
            fn into_val(self) -> Self::Item {
                let ($($name),*) = self;
                $ret
            }
        }
    }
}

tuple_impl!(0, T0, _t0, [ T0, T1 ], [ _t0, _t1 ]);
tuple_impl!(1, T1, _t1, [ T0, T1 ], [ _t0, _t1 ]);
tuple_impl!(0, T0, _t0, [ T0, T1, T2 ], [ _t0, _t1, _t2 ]);
tuple_impl!(1, T1, _t1, [ T0, T1, T2 ], [ _t0, _t1, _t2 ]);
tuple_impl!(2, T2, _t2, [ T0, T1, T2 ], [ _t0, _t1, _t2 ]);
tuple_impl!(0, T0, _t0, [ T0, T1, T2, T3 ], [ _t0, _t1, _t2, _t3 ]);
tuple_impl!(1, T1, _t1, [ T0, T1, T2, T3 ], [ _t0, _t1, _t2, _t3 ]);
tuple_impl!(2, T2, _t2, [ T0, T1, T2, T3 ], [ _t0, _t1, _t2, _t3 ]);
tuple_impl!(3, T3, _t3, [ T0, T1, T2, T3 ], [ _t0, _t1, _t2, _t3 ]);
tuple_impl!(0, T0, _t0, [ T0, T1, T2, T3, T4 ], [ _t0, _t1, _t2, _t3, _t4 ]);
tuple_impl!(1, T1, _t1, [ T0, T1, T2, T3, T4 ], [ _t0, _t1, _t2, _t3, _t4 ]);
tuple_impl!(2, T2, _t2, [ T0, T1, T2, T3, T4 ], [ _t0, _t1, _t2, _t3, _t4 ]);
tuple_impl!(3, T3, _t3, [ T0, T1, T2, T3, T4 ], [ _t0, _t1, _t2, _t3, _t4 ]);
tuple_impl!(4, T4, _t4, [ T0, T1, T2, T3, T4 ], [ _t0, _t1, _t2, _t3, _t4 ]);
tuple_impl!(0, T0, _t0, [ T0, T1, T2, T3, T4, T5 ], [ _t0, _t1, _t2, _t3, _t4, _t5 ]);
tuple_impl!(1, T1, _t1, [ T0, T1, T2, T3, T4, T5 ], [ _t0, _t1, _t2, _t3, _t4, _t5 ]);
tuple_impl!(2, T2, _t2, [ T0, T1, T2, T3, T4, T5 ], [ _t0, _t1, _t2, _t3, _t4, _t5 ]);
tuple_impl!(3, T3, _t3, [ T0, T1, T2, T3, T4, T5 ], [ _t0, _t1, _t2, _t3, _t4, _t5 ]);
tuple_impl!(4, T4, _t4, [ T0, T1, T2, T3, T4, T5 ], [ _t0, _t1, _t2, _t3, _t4, _t5 ]);
tuple_impl!(5, T5, _t5, [ T0, T1, T2, T3, T4, T5 ], [ _t0, _t1, _t2, _t3, _t4, _t5 ]);

pub trait IterExt: Iterator {
    fn item_in_tuple<const N: usize>(self) -> impl Iterator<Item = <Self::Item as Tuple<N>>::Item>
    where
        Self: Sized,
        Self::Item: Tuple<N>,
    {
        self.map(|x| x.into_val())
    }
}
