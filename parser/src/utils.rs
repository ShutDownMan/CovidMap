use pgvector::Vector;
use std::fmt;

// pub struct PgVec(pub Vec<f32>);

// impl fmt::Display for Vector {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "ARRAY[{0}]",
//             self.0
//                 .iter()
//                 .map(|x| { x.to_string() })
//                 .collect::<Vec<String>>()
//                 .join(", ")
//         )
//     }
// }

// impl fmt::Debug for Vector {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "ARRAY[{0}]",
//             self.0
//                 .iter()
//                 .map(|x| { x.to_string() })
//                 .collect::<Vec<String>>()
//                 .join(", ")
//         )
//     }
// }
