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
            vec![Trip::new(1, vec![10, 20, 30, 40, 50])],
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
                vec![Trip::new(1, vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![5, 6, 2, 7, 8],
                vec![Trip::new(2, vec![20, 40, 60, 70, 80])],
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
                vec![Trip::new(1, vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![0, 5, 6, 7, 4],
                vec![Trip::new(2, vec![7, 20, 40, 60, 73])],
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
                vec![Trip::new(1, vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![0, 5, 6, 7],
                vec![Trip::new(2, vec![7, 10, 20, 30])],
            ),
            Route::new(
                false,
                vec![7, 8, 9, 4],
                vec![Trip::new(3, vec![35, 45, 55, 65])],
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
                vec![Trip::new(1, vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![0, 5, 6, 7],
                vec![Trip::new(2, vec![7, 15, 20, 25])],
            ),
            Route::new(
                false,
                vec![7, 8, 9, 4],
                vec![Trip::new(3, vec![30, 35, 40, 45])],
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
                vec![Trip::new(1, vec![10, 20, 30, 40, 50])],
            ),
            Route::new(
                false,
                vec![5, 6, 7, 8],
                vec![Trip::new(2, vec![10, 20, 40, 60])],
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

#[test]
fn circle() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0]),
        ],
        vec![Route::new(
            true,
            vec![0, 1, 2, 3, 4],
            vec![
                Trip::new(1, vec![10, 20, 30, 40, 50, 60]),
                Trip::new(2, vec![30, 40, 50, 60, 70, 80]),
                Trip::new(3, vec![60, 70, 80, 90, 100, 110]),
            ],
        )],
        vec![vec![]; 5],
    );
    let platforms = Platforms::from(Walking::from([(1, 5)]), Walking::from([(3, 10)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 2., y: 0.},
                coord! {x: 3., y: 0.},
                coord! {x: 4., y: 0.},
            ],
            Some(0),
        )],
        50,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn circle_on_seam() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0]),
            Platform::new(Point::new(0., 2.), vec![0]),
            Platform::new(Point::new(0., 3.), vec![0]),
            Platform::new(Point::new(0., 4.), vec![0]),
            Platform::new(Point::new(0., 5.), vec![0]),
        ],
        vec![Route::new(
            true,
            vec![0, 1, 2, 3, 4],
            vec![
                Trip::new(1, vec![10, 20, 30, 40, 50, 60]),
                Trip::new(2, vec![30, 40, 50, 60, 70, 80]),
                Trip::new(3, vec![60, 70, 80, 90, 100, 110]),
            ],
        )],
        vec![vec![]; 5],
    );
    let platforms = Platforms::from(Walking::from([(3, 5)]), Walking::from([(1, 5)]));
    let mut searcher = Searcher::new(&map, platforms);
    // TODO - how to join two parts?
    let expected: Vec<Path> = vec![Path::new(
        vec![
            Part::new(vec![coord! {x: 4., y: 0.}, coord! {x: 5., y: 0.}], Some(0)),
            Part::new(
                vec![
                    coord! {x: 5., y: 0.},
                    coord! {x: 1., y: 0.},
                    coord! {x: 2., y: 0.},
                ],
                Some(0),
            ),
        ],
        75,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn bidirectional_circle() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0, 1]),
            Platform::new(Point::new(0., 2.), vec![0, 1]),
            Platform::new(Point::new(0., 3.), vec![0, 1]),
            Platform::new(Point::new(0., 4.), vec![0, 1]),
            Platform::new(Point::new(0., 5.), vec![0, 1]),
        ],
        vec![
            Route::new(
                true,
                vec![0, 1, 2, 3, 4],
                vec![
                    Trip::new(1, vec![10, 20, 30, 40, 50, 60]),
                    Trip::new(2, vec![30, 40, 50, 60, 70, 80]),
                    Trip::new(3, vec![60, 70, 80, 90, 100, 110]),
                ],
            ),
            Route::new(
                true,
                vec![4, 3, 2, 1, 0],
                vec![
                    Trip::new(4, vec![15, 25, 35, 45, 55, 65]),
                    Trip::new(5, vec![35, 45, 55, 65, 75, 85]),
                    Trip::new(6, vec![65, 75, 85, 95, 105, 115]),
                ],
            ),
        ],
        vec![vec![]; 5],
    );
    let platforms = Platforms::from(Walking::from([(3, 5)]), Walking::from([(1, 5)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![Part::new(
            vec![
                coord! {x: 4., y: 0.},
                coord! {x: 3., y: 0.},
                coord! {x: 2., y: 0.},
            ],
            Some(1),
        )],
        50,
    )];
    assert_eq!(expected, searcher.run(1));
}

#[test]
fn bidirectional_circle_on_seam() {
    let map = PublicTransport::new(
        vec![
            Platform::new(Point::new(0., 1.), vec![0, 1]),
            Platform::new(Point::new(0., 2.), vec![0, 1]),
            Platform::new(Point::new(0., 3.), vec![0, 1]),
            Platform::new(Point::new(0., 4.), vec![0, 1]),
            Platform::new(Point::new(0., 5.), vec![0, 1]),
        ],
        vec![
            Route::new(
                true,
                vec![0, 1, 2, 3, 4],
                vec![
                    Trip::new(1, vec![10, 30, 50, 70, 90, 110]),
                    Trip::new(2, vec![60, 80, 100, 120, 140, 160]),
                    Trip::new(3, vec![110, 130, 150, 170, 190, 210]),
                ],
            ),
            Route::new(
                true,
                vec![4, 3, 2, 1, 0],
                vec![
                    Trip::new(4, vec![15, 25, 35, 45, 55, 65]),
                    Trip::new(5, vec![35, 45, 55, 65, 75, 85]),
                    Trip::new(6, vec![65, 75, 85, 95, 105, 115]),
                ],
            ),
        ],
        vec![vec![]; 5],
    );
    let platforms = Platforms::from(Walking::from([(1, 5)]), Walking::from([(3, 5)]));
    let mut searcher = Searcher::new(&map, platforms);
    let expected: Vec<Path> = vec![Path::new(
        vec![
            Part::new(vec![coord! {x: 2., y: 0.}, coord! {x: 1., y: 0.}], Some(1)),
            Part::new(
                vec![
                    coord! {x: 1., y: 0.},
                    coord! {x: 5., y: 0.},
                    coord! {x: 4., y: 0.},
                ],
                Some(1),
            ),
        ],
        80,
    )];
    assert_eq!(expected, searcher.run(26));
}
