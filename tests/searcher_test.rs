use geo_types::coord;

use tranet::{
    map::{Platform, Point, PublicTransport, Route, Trip},
    path::{Part, Path},
    platforms::{Platforms, Walking},
    searcher::Searcher,
};

#[test]
fn one_route() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0]),
        ],
        vec![Route::new(
            false,
            vec![0, 1, 2, 3, 4],
            vec![Trip::new(vec![10, 20, 30, 40, 50])],
        )],
        vec![vec![], vec![], vec![], vec![], vec![]],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(4, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 1.0, y: 0.0},
                coord! {x: 2.0, y: 0.0},
                coord! {x: 3.0, y: 0.0},
                coord! {x: 4.0, y: 0.0},
                coord! {x: 5.0, y: 0.0},
            ],
            Some(0),
        )],
        60,
    )];
    assert_eq!(expected, searcher.run(1));
}
