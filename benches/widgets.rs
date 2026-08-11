use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use termlayout::widgets::{
    Cell, CellWidth, Filler, Frame, FrameDecoration, Horizontal, Lines, List, Menu, MenuItem,
    Paragraph, Table, TableColumn, TableDecoration, Tree, TreeDecoration, TreeNode, Vertical,
};
use termlayout::{Dimension, Layout, LayoutOptions, MeasureMode, RcLayout, WrapMode};

// ── shared content ────────────────────────────────────────────────────────────

const SHORT: &str = "Hello, world!";

const MEDIUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
    Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.";

const LONG: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
    Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
    nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
    reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
    pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa \
    qui officia deserunt mollit anim id est laborum. \
    Sed ut perspiciatis unde omnis iste natus error sit voluptatem \
    accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae \
    ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt \
    explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut \
    odit aut fugit, sed quia consequuntur magni dolores eos qui ratione \
    voluptatem sequi nesciunt.";

// ── Lines ─────────────────────────────────────────────────────────────────────

fn bench_lines(c: &mut Criterion) {
    let mut g = c.benchmark_group("lines");

    for (label, content) in [("short", SHORT), ("medium", MEDIUM), ("long", LONG)] {
        let widget = Lines::left(content);
        g.bench_with_input(BenchmarkId::new("measure", label), label, |b, _| {
            b.iter(|| black_box(widget.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout", label), label, |b, _| {
            b.iter(|| black_box(format!("{}", widget.layout(80))));
        });
    }

    // alignment variants on MEDIUM content
    let left = Lines::left(MEDIUM);
    let center = Lines::center(MEDIUM);
    let right = Lines::right(MEDIUM);
    g.bench_function("layout/left", |b| {
        b.iter(|| black_box(format!("{}", left.layout(80))));
    });
    g.bench_function("layout/center", |b| {
        b.iter(|| black_box(format!("{}", center.layout(80))));
    });
    g.bench_function("layout/right", |b| {
        b.iter(|| black_box(format!("{}", right.layout(80))));
    });

    g.finish();
}

// ── Paragraph ─────────────────────────────────────────────────────────────────

fn bench_paragraph(c: &mut Criterion) {
    let mut g = c.benchmark_group("paragraph");

    for (label, content) in [("short", SHORT), ("medium", MEDIUM), ("long", LONG)] {
        let widget = Paragraph::left(content);
        g.bench_with_input(BenchmarkId::new("measure", label), label, |b, _| {
            b.iter(|| black_box(widget.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout", label), label, |b, _| {
            b.iter(|| black_box(format!("{}", widget.layout(80))));
        });
    }

    g.finish();
}

// ── Filler ────────────────────────────────────────────────────────────────────

fn bench_filler(c: &mut Criterion) {
    let mut g = c.benchmark_group("filler");
    let dim = Dimension::new(80, 24);
    let options = LayoutOptions::default().with_dim(dim);

    let once = Filler::once("*");
    let horiz = Filler::horizontal("abc");
    let vert = Filler::vertical("-");
    let both = Filler::both(".");

    g.bench_function("measure/once", |b| {
        b.iter(|| black_box(once.measure(MeasureMode::fixed_width(80, WrapMode::default()))));
    });
    g.bench_function("measure/horizontal", |b| {
        b.iter(|| black_box(horiz.measure(MeasureMode::fixed_width(80, WrapMode::default()))));
    });
    g.bench_function("measure/vertical", |b| {
        b.iter(|| black_box(vert.measure(MeasureMode::fixed_width(80, WrapMode::default()))));
    });
    g.bench_function("measure/both", |b| {
        b.iter(|| black_box(both.measure(MeasureMode::fixed_width(80, WrapMode::default()))));
    });

    g.bench_function("layout/once", |b| {
        b.iter(|| black_box(format!("{}", once.layout_strict(options))));
    });
    g.bench_function("layout/horizontal", |b| {
        b.iter(|| black_box(format!("{}", horiz.layout_strict(options))));
    });
    g.bench_function("layout/vertical", |b| {
        b.iter(|| black_box(format!("{}", vert.layout_strict(options))));
    });
    g.bench_function("layout/both", |b| {
        b.iter(|| black_box(format!("{}", both.layout_strict(options))));
    });

    g.finish();
}

// ── Cell ──────────────────────────────────────────────────────────────────────

fn bench_cell(c: &mut Criterion) {
    let mut g = c.benchmark_group("cell");
    let content: RcLayout = Lines::left(MEDIUM).into();
    let dim = Dimension::new(40, 10);
    let options = LayoutOptions::default().with_dim(dim);

    let minimal = Cell::minimal(content.clone());
    let fill = Cell::fill(content.clone());
    let fixed = Cell::of(content).with_width(CellWidth::Fixed(40));

    g.bench_function("measure/minimal", |b| {
        b.iter(|| black_box(minimal.measure(MeasureMode::pref_width(40, WrapMode::Wrap))));
    });
    g.bench_function("measure/fill", |b| {
        b.iter(|| black_box(fill.measure(MeasureMode::fixed_width(40, WrapMode::Wrap))));
    });
    g.bench_function("measure/fixed", |b| {
        b.iter(|| black_box(fixed.measure(MeasureMode::fixed_width(40, WrapMode::Wrap))));
    });

    g.bench_function("layout/minimal", |b| {
        b.iter(|| black_box(format!("{}", minimal.layout_strict(options))));
    });
    g.bench_function("layout/fill", |b| {
        b.iter(|| black_box(format!("{}", fill.layout_strict(options))));
    });
    g.bench_function("layout/fixed", |b| {
        b.iter(|| black_box(format!("{}", fixed.layout_strict(options))));
    });

    g.finish();
}

// ── Frame ─────────────────────────────────────────────────────────────────────

fn bench_frame(c: &mut Criterion) {
    let mut g = c.benchmark_group("frame");

    let boxed = Frame::new(
        FrameDecoration::boxed(),
        Some("Title".to_string()),
        Paragraph::left(MEDIUM),
    );
    let double = Frame::new(
        FrameDecoration::double_boxed(),
        Some("Title".to_string()),
        Paragraph::left(MEDIUM),
    );

    g.bench_function("measure/boxed", |b| {
        b.iter(|| black_box(boxed.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
    });
    g.bench_function("measure/double", |b| {
        b.iter(|| black_box(double.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
    });

    g.bench_function("layout/boxed", |b| {
        b.iter(|| black_box(format!("{}", boxed.layout(80))));
    });
    g.bench_function("layout/double", |b| {
        b.iter(|| black_box(format!("{}", double.layout(80))));
    });

    g.finish();
}

// ── Horizontal ────────────────────────────────────────────────────────────────

fn bench_horizontal(c: &mut Criterion) {
    let mut g = c.benchmark_group("horizontal");

    let make_horizontal = |n: usize| {
        let cells: Vec<Cell> = (0..n)
            .map(|i| Cell::fill(Lines::left(format!("col{i}"))))
            .collect();
        Horizontal::new(cells, None)
    };

    for n in [2usize, 5, 10] {
        let widget = make_horizontal(n);
        g.bench_with_input(BenchmarkId::new("measure", n), &n, |b, _| {
            b.iter(|| black_box(widget.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout", n), &n, |b, _| {
            b.iter(|| black_box(format!("{}", widget.layout(80))));
        });
    }

    g.finish();
}

// ── Vertical ──────────────────────────────────────────────────────────────────

fn bench_vertical(c: &mut Criterion) {
    let mut g = c.benchmark_group("vertical");

    let make_vertical = |n: usize| {
        let items: Vec<RcLayout> = (0..n)
            .map(|i| Lines::left(format!("line {i}: {MEDIUM}")).into())
            .collect();
        Vertical::new(items)
    };

    for n in [2usize, 5, 10] {
        let widget = make_vertical(n);
        g.bench_with_input(BenchmarkId::new("measure", n), &n, |b, _| {
            b.iter(|| black_box(widget.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout", n), &n, |b, _| {
            b.iter(|| black_box(format!("{}", widget.layout(80))));
        });
    }

    g.finish();
}

// ── Table ─────────────────────────────────────────────────────────────────────

fn make_table(rows: usize, cols: usize) -> Table {
    let columns: Vec<TableColumn> = (0..cols)
        .map(|i| TableColumn::DEFAULT.with_header(Lines::left(format!("H{i}"))))
        .collect();
    let cells: Vec<Vec<RcLayout>> = (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| Lines::left(format!("R{r}C{c}")).into())
                .collect()
        })
        .collect();
    Table::new(TableDecoration::boxed_grid(), columns, cells)
}

fn bench_table(c: &mut Criterion) {
    let mut g = c.benchmark_group("table");

    for (label, rows, cols) in [("2x2", 2, 2), ("4x4", 4, 4), ("8x4", 8, 4)] {
        let widget = make_table(rows, cols);
        g.bench_with_input(BenchmarkId::new("measure", label), label, |b, _| {
            b.iter(|| black_box(widget.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout", label), label, |b, _| {
            b.iter(|| black_box(format!("{}", widget.layout(80))));
        });
    }

    g.finish();
}

// ── Tree ──────────────────────────────────────────────────────────────────────

fn make_tree(depth: usize, branching: usize) -> Tree {
    fn make_node(label: &str, depth: usize, branching: usize) -> TreeNode {
        if depth == 0 {
            return TreeNode::leaf(Lines::left(label));
        }
        let children: Vec<TreeNode> = (0..branching)
            .map(|i| make_node(&format!("{label}.{i}"), depth - 1, branching))
            .collect();
        TreeNode::new(Lines::left(label), children)
    }

    let root = make_node("root", depth, branching);
    Tree::new(TreeDecoration::ticks('▸'), root, true)
}

fn bench_tree(c: &mut Criterion) {
    let mut g = c.benchmark_group("tree");

    let shallow = make_tree(1, 4); // 1 root + 4 leaves
    let deep = make_tree(3, 3); // 1 + 3 + 9 + 27 = 40 nodes

    g.bench_function("measure/shallow", |b| {
        b.iter(|| black_box(shallow.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
    });
    g.bench_function("measure/deep", |b| {
        b.iter(|| black_box(deep.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
    });

    g.bench_function("layout/shallow", |b| {
        b.iter(|| black_box(format!("{}", shallow.layout(80))));
    });
    g.bench_function("layout/deep", |b| {
        b.iter(|| black_box(format!("{}", deep.layout(80))));
    });

    g.finish();
}

// ── List ──────────────────────────────────────────────────────────────────────

fn bench_list(c: &mut Criterion) {
    let mut g = c.benchmark_group("list");

    let make_items = |n: usize| -> Vec<RcLayout> {
        (0..n)
            .map(|i| Lines::left(format!("Item {i}: {SHORT}")).into())
            .collect()
    };

    for n in [5usize, 20, 50] {
        let numbered = List::numbered(make_items(n));
        let fixed = List::fixed(make_items(n));

        g.bench_with_input(BenchmarkId::new("measure/numbered", n), &n, |b, _| {
            b.iter(|| black_box(numbered.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout/numbered", n), &n, |b, _| {
            b.iter(|| black_box(format!("{}", numbered.layout(80))));
        });
        g.bench_with_input(BenchmarkId::new("measure/fixed", n), &n, |b, _| {
            b.iter(|| black_box(fixed.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
        });
        g.bench_with_input(BenchmarkId::new("layout/fixed", n), &n, |b, _| {
            b.iter(|| black_box(format!("{}", fixed.layout(80))));
        });
    }

    g.finish();
}

// ── Menu ──────────────────────────────────────────────────────────────────────

fn bench_menu(c: &mut Criterion) {
    let mut g = c.benchmark_group("menu");

    let items: Vec<MenuItem> = vec![
        MenuItem::new('n', Lines::left("New file")),
        MenuItem::new('o', Lines::left("Open file")),
        MenuItem::new('s', Lines::left("Save")),
        MenuItem::new('x', Lines::left("Exit")),
    ];

    let menu = Menu::new(items);

    g.bench_function("measure", |b| {
        b.iter(|| black_box(menu.measure(MeasureMode::pref_width(80, WrapMode::Wrap))));
    });
    g.bench_function("layout", |b| {
        b.iter(|| black_box(format!("{}", menu.layout(80))));
    });

    g.finish();
}

// ── criterion setup ───────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_lines,
    bench_paragraph,
    bench_filler,
    bench_cell,
    bench_frame,
    bench_horizontal,
    bench_vertical,
    bench_table,
    bench_tree,
    bench_list,
    bench_menu,
);
criterion_main!(benches);
