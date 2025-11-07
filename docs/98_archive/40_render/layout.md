# Layout

**Intrinsic node**: A node that has an intrinsic size (text size, image size, etc.)

**Content-sized node**: A node that has a size defined by its content (column, row, etc.) All intrinsic nodes are content-sized.

**Multi-pass layout**: A layout algorithm that lays out non-content-sized nodes more than once to find the best layout configuration.

Starting point of the layout process:

1. ART has all the supplied render data.
2. ART contains all nodes.
3. The actual UI has a width and a height (measured in a platform-dependent way).

We start with the first viewport and perform a general, mostly algorithm antagonistic process.

Horizonal and vertical layout calculations are logically independent of each other (they might be
implemented in the same function to avoid looping over structures twice).

## Measurement and layout

The layout system performs measurement and layout in a single pass. These two cannot be separated
theoretically because measurement of container nodes requires performing the actual layout 
in the general case. 

For example, you **cannot** deterministically measure a row of items without actually performing
the layout counting gap, item sizes, surroundings, spacing all together.

If the layout calculated during measurement is good enough, it is pointless to do it again.

## General layout flow

1. Measure all content-sized nodes.
    1. Intrinsic nodes simply return their `intrinsic_size`.
    2. Container nodes perform a layout pass and return with `content_size + surrounding`.
2. Perform the layout adjustment loop:  
    1. Distribute available space among children.
    2. Lay out all non-content-sized children.
    3. If the result is acceptable, finish the loop.
    4. Adjust parameters, run the loop again.
3. Position all children.
4. Generate layout patches.

## Adjustment loop

In the majority of cases the adjustment loop is lowered into a single pass. For example, a basic column
layout without any specific fill strategy simply puts children after each other and goes on.

In some cases, `resize_to_max` for example, there is more than one pass:

1. The first pass measures all children as they would have unlimited space. 
2. Then the required space is calculated (if alignment is `baseline`, then taking it into account).
3. The second pass lays out all children with the calculated space.

