/// Create an enum that can be iterated over with tab/shift-tab
#[macro_export]
macro_rules! form_fields {
    ($name:ident, $($variant:ident),*) => {
        #[derive(Debug, Clone, Copy)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            const FIELDS: &'static [$name] = &[
                $($name::$variant),*
            ];

            fn index(&self) -> usize {
                match self {
                    $(Self::$variant => $name::$variant as usize),*
                }
            }

            /// Rotate through the options (e.g. with tab)
            fn next(&self) -> Self {
                Self::FIELDS[(self.index() + 1) % Self::FIELDS.len()]
            }

            /// Rotate through the options in reverse (e.g. with shift-tab)
            fn prev(&self) -> Self {
                Self::FIELDS[(self.index() + Self::FIELDS.len() - 1) % Self::FIELDS.len()]
            }
        }
    };
}
