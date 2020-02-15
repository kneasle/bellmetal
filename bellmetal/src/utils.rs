use crate::Change;

pub fn closure (change : Change) -> Vec<Change> {
    let mut vec : Vec<Change> = Vec::with_capacity (change.stage ().as_usize ());

    let rounds = Change::rounds (change.stage ());
    let mut accum = change.clone ();

    vec.push (rounds.clone ());

    while accum != rounds {
        vec.push (accum.clone ());

        accum = accum * change.clone ();
    }

    vec
}

#[cfg(test)]
mod utils_tests {
    use crate::utils;
    use crate::change::Change;

    #[test]
    fn closure () {
        assert_eq! (
            utils::closure (Change::from ("13425678")),
            vec! [
                Change::from ("12345678"),
                Change::from ("13425678"),
                Change::from ("14235678")
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("87654321")),
            vec! [
                Change::from ("12345678"),
                Change::from ("87654321")
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("1")),
            vec! [
                Change::from ("1"),
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("123456789")),
            vec! [
                Change::from ("123456789"),
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("4321675")),
            vec! [
                Change::from ("1234567"),
                Change::from ("4321675"),
                Change::from ("1234756"),
                Change::from ("4321567"),
                Change::from ("1234675"),
                Change::from ("4321756")
            ]
        );
        
        assert_eq! (
            utils::closure (Change::from ("")),
            vec! [
                Change::from (""),
            ]
        );
    }
}
