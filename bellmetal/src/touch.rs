use crate::types::{ Stage, Bell, Number };

pub struct Row {
    pub index : u32
}

struct Touch {
    pub stage : Stage,
    pub length : Number,
    bells : Vec<Bell>,
    rows : Vec<Row>
}

impl Touch {
    fn to_string (&self) -> String {
        let stage = self.stage.as_number ();

        let mut s = String::with_capacity ((stage * self.length + self.length - 1) as usize);

        for i in 0..self.rows.len () {
            for j in 0..stage as usize {
                s.push (self.bells [i * (stage as usize) + j].as_char ());
            }
            
            if i != self.rows.len () - 1 {
                s.push ('\n');
            }
        }

        s
    }
}

impl From<&str> for Touch {
    fn from (string : &str) -> Touch {
        let (stage, length) = {
            let mut length = 0;
            let mut potential_stage = None;

            for line in string.lines () {
                match potential_stage {
                    Some (s) => {
                        if s != line.len () {
                            panic! ("Every row of a stage must be the same length");
                        }
                    }
                    None => { potential_stage = Some (line.len ()); }
                }

                length += 1;
            }

            match potential_stage {
                Some (s) => { (s, length) }
                None => { panic! ("Cannot create an empty touch"); }
            }
        };
        
        let bells = {
            let mut bells : Vec<Bell> = Vec::with_capacity (length * stage);
            
            for line in string.lines () {
                for c in line.chars () {
                    bells.push (Bell::from (c));
                }
            }

            bells
        };

        let mut rows : Vec<Row> = Vec::with_capacity (length);
        
        for i in 0..length {
            rows.push (Row {
                index : i as Number
            });
        }
        
        Touch { 
            stage : Stage::from (stage),
            length : length as Number,
            bells : bells,
            rows : rows
        }
    }
}

#[cfg(test)]
mod touch_tests {
    use crate::touch::Touch;

    #[test]
    fn string_conversions () {
        for s in vec! [
            "123456\n214365\n123456",
            "123\n213\n231\n321\n312\n132\n123",
            "1"
        ] {
            assert_eq! (Touch::from (s).to_string (), s);
        }
    }
}
