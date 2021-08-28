pub fn cross_product<T: Clone>(mut i: Vec<Vec<T>>) -> Vec<Vec<T>> {
    if i.len() == 1 {
        i.pop()
            .expect("it's there")
            .into_iter()
            .map(|i| vec![i])
            .collect()
    } else {
        let mut new_i = i;
        let friend = new_i.pop().expect("we checked it");
        let cp = cross_product(new_i);
        let mut build = vec![];
        for x in friend {
            for item in cp.iter() {
                let mut ni = item.clone();
                ni.push(x.clone());
                build.push(ni)
            }
        }
        build
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_crossproduct() {
        let values = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let mut cp = cross_product(values.clone());
        let mut from_crate = values
            .into_iter()
            .multi_cartesian_product()
            .collect::<Vec<_>>();
        cp.sort();
        from_crate.sort();
        assert_eq!(cp, from_crate);

        for x in cp {
            eprintln!("{:?}", x)
        }
        for x in from_crate {
            eprintln!("{:?}", x)
        }
    }
}
