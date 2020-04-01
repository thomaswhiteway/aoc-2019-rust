#[derive(Clone, Copy)]
struct Password(u32);

impl Password {
    fn is_valid(&self) -> bool {
        let string = self.0.to_string();
        for (c1, c2) in string.chars().zip(string.chars().skip(1)) {
            if c2 < c1 {
                return false;
            } 
        }

        let mut have_pair = false;

        let mut current_char = string.chars().nth(0).unwrap();
        let mut group_size = 1;
        for c in string.chars().skip(1) {
            if c != current_char {
                if group_size == 2 {
                    have_pair = true;
                }

                current_char = c;
                group_size = 1;
            } else {
                group_size += 1;
            }
        }

        if group_size == 2 {
            have_pair = true;
        }

        have_pair
    }
}

fn main() {
    let num_passwords = (134792..=675810).map(Password).filter(Password::is_valid).count();

    println!("{} valid passwords", num_passwords);
}
