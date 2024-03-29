use geo_types::coord;

use tranet::{
    map::{Passage, Platform, Point, PublicTransport, Route, Trip},
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
        vec![vec![]; 5],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(4, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 1., y: 0.},
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 0.},
                coord! {x: 4., y: 0.},
                coord! {x: 5., y: 0.},
            ],
            Some(0),
        )],
        60,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn two_cross_route() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0, 1]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0]),
            Platform::new(Point::new(0., 10.), vec![1]),
            Platform::new(Point::new(0., 11.), vec![1]),
            Platform::new(Point::new(0., 12.), vec![1]),
            Platform::new(Point::new(0., 13.), vec![1]),
        ],
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4],
                vec![Trip::new(vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![5, 6, 2, 7, 8],
                vec![Trip::new(vec![20, 40, 60, 70, 80])],
            ),
        ],
        vec![vec![]; 9],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(4, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 1., y: 0.},
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 0.},
                coord! {x: 4., y: 0.},
                coord! {x: 5., y: 0.},
            ],
            Some(0),
        )],
        60,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn two_parallel_route() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0, 1]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0, 1]),
            Platform::new(Point::new(0., 10.), vec![1]),
            Platform::new(Point::new(0., 11.), vec![1]),
            Platform::new(Point::new(0., 12.), vec![1]),
        ],
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4],
                vec![Trip::new(vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![0, 5, 6, 7, 4],
                vec![Trip::new(vec![7, 20, 40, 60, 73])],
            ),
        ],
        vec![vec![]; 8],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(4, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 1., y: 0.},
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 0.},
                coord! {x: 4., y: 0.},
                coord! {x: 5., y: 0.},
            ],
            Some(0),
        )],
        60,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn direct_faster() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0, 1]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0, 2]),
            Platform::new(Point::new(0., 10.), vec![1]),
            Platform::new(Point::new(0., 11.), vec![1]),
            Platform::new(Point::new(0., 12.), vec![1, 2]),
            Platform::new(Point::new(0., 20.), vec![2]),
            Platform::new(Point::new(0., 21.), vec![2]),
        ],
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4],
                vec![Trip::new(vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![0, 5, 6, 7],
                vec![Trip::new(vec![7, 10, 20, 30])],
            ),
            Route::new(
                false,
                vec![7, 8, 9, 4],
                vec![Trip::new(vec![35, 45, 55, 65])],
            ),
        ],
        vec![vec![]; 10],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(4, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 1., y: 0.},
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 0.},
                coord! {x: 4., y: 0.},
                coord! {x: 5., y: 0.},
            ],
            Some(0),
        )],
        60,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn direct_slower() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0, 1]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0, 2]),
            Platform::new(Point::new(0., 10.), vec![1]),
            Platform::new(Point::new(0., 11.), vec![1]),
            Platform::new(Point::new(0., 12.), vec![1, 2]),
            Platform::new(Point::new(0., 20.), vec![2]),
            Platform::new(Point::new(0., 21.), vec![2]),
        ],
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4],
                vec![Trip::new(vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![0, 5, 6, 7],
                vec![Trip::new(vec![7, 15, 20, 25])],
            ),
            Route::new(
                false,
                vec![7, 8, 9, 4],
                vec![Trip::new(vec![30, 35, 40, 45])],
            ),
        ],
        vec![vec![]; 10],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(4, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![
        Path::new(
            vec![Part::new(
                vec![
                    coord! {x: 1., y: 0.},
                    coord! {x: 2., y: 0.},
                    coord! {x: 3., y: 0.},
                    coord! {x: 4., y: 0.},
                    coord! {x: 5., y: 0.},
                ],
                Some(0),
            )],
            60,
        ),
        Path::new(
            vec![
                Part::new(
                    vec![
                        coord! {x: 1., y: 0.},
                        coord! {x: 10., y: 0.},
                        coord! {x: 11., y: 0.},
                        coord! {x: 12., y: 0.},
                    ],
                    Some(1),
                ),
                Part::new(
                    vec![
                        coord! {x: 12., y: 0.},
                        coord! {x: 20., y: 0.},
                        coord! {x: 21., y: 0.},
                        coord! {x: 5., y: 0.},
                    ],
                    Some(2),
                ),
            ],
            55,
        ),
    ];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn do_transfer() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0]),
            Platform::new(Point::new(0., 10.), vec![1]),
            Platform::new(Point::new(0., 11.), vec![1]),
            Platform::new(Point::new(0., 12.), vec![1]),
            Platform::new(Point::new(0., 13.), vec![1]),
        ],
        vec![
            Route::new(
                false,
                vec![0, 1, 2, 3, 4],
                vec![Trip::new(vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![5, 6, 7, 8],
                vec![Trip::new(vec![10, 20, 40, 60])],
            ),
        ],
        vec![
            vec![],
            vec![],
            vec![Passage::new(7, 5)],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ],
    );
    let platforms = Platforms::from(Walking::from([(0, 5)]), Walking::from([(8, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![
            Part::new(
                vec![
                    coord! {x: 1., y: 0.},
                    coord! {x: 2., y: 0.},
                    coord! {x: 3., y: 0.},
                ],
                Some(0),
            ),
            Part::new(vec![coord! {x: 3., y: 0.}, coord! {x: 12., y: 0.}], None),
            Part::new(
                vec![coord! {x: 12., y: 0.}, coord! {x: 13., y: 0.}],
                Some(1),
            ),
        ],
        70,
    )];
    assert_eq!(expected, searcher.run(1));
}
