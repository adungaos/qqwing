#[derive(Debug, Clone, PartialEq)]
pub enum Symmetry {
    NONE,
    ROTATE90,
    ROTATE180,
    MIRROR,
    FLIP,
    RANDOM,
}

// 	pub fn get(String s) {
// 		if (s == null) return null;
// 		try {
// 			s = s.toUpperCase(Locale.ENGLISH);
// 			return valueOf(s);
// 		} catch (IllegalArgumentException aix) {
// 			return null;
// 		}
// 	}

// 	public String getName() {
// 		String name = toString();
// 		return name.substring(0, 1) + name.substring(1).toLowerCase(Locale.ENGLISH);
// 	}
// }
