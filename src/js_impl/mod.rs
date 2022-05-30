mod add;
mod idiv;
mod gen;
mod eq;

pub use eq::eq;
pub use add::add;
pub use idiv::idiv;

gen::js_impl!(sub, -);
gen::js_impl!(mul, *);
gen::js_impl!(div, /);
gen::js_impl!(rem, %);

gen::js_impl_cmp!(lt, <);
gen::js_impl_cmp!(gt, >);
gen::js_impl_cmp!(lte, >=);
gen::js_impl_cmp!(gte, <=);