# Version 0.1.1

- Reimplementation of the sizing system: instead of providing `pref_dim` and `min_dim` to determine the size of the 
  layout, we use the `measure` method instead which returns `Measurements`. Since the `Measurments` are not just the
  dimension of the layout but contain also cached sizing information about subordinated elements, the performance is 
  improved at least for complex widgets like table, vertical a.s.o.
- Benchmarks added
- Minor bugfixes

# Version 0.1.0

Initial release. Providing the following features:

- Basic `Layout` framework
- `Lines` widget
- `Paragraph` widget
- `Filler` widget
- `Cell` widget
- `Frame` widget
- `Vertical` widget
- `Horizontal` widget
- `Table` widget
- `List` widget
- `Menu` widget
- `Markdown` widget (requires `markdown` feature)
- `Tree` widget