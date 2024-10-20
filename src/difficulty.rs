#[derive(Debug)]
pub enum Difficulty {
    UNKNOWN,
    SIMPLE,
    EASY,
    INTERMEDIATE,
    EXPERT,
}

// 	public static Difficulty get(String s) {
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
