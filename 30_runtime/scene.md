# Scene, Channels, Viewports, and Layers

The **scene** is the root of the rendered interface. It organizes all visual content 
into **channels**, **viewports**, and **layers**.

## Channels

**Channels** represent global z-order bands within the scene. They define the top-level stacking 
order and contain one or more **viewports**.

| Channel     | Purpose                                           | Captures Input? | Notes                                  |
|-------------|---------------------------------------------------|-----------------|----------------------------------------|
| **main**    | Primary application content.                      | ✅ Yes           | Base of the scene.                     |
| **modal**   | Dialogs or sheets that suspend interaction below. | ✅ Yes           | Above main; blocks lower channels.     |
| **snack**   | Non-blocking feedback (toasts, banners).          | ❌               | Above modal; temporary feedback layer. |
| **tooltip** | Hover hints, popovers, contextual info.           | ❌               | Always on top; most transient.         |

Render and event order (bottom → top):

```
main < modal < snack < tooltip
```

Channels do not clip or scroll on their own. Each channel is a logical container for **viewports** that 
define local coordinate spaces and scrolling.

## Viewports

A **viewport** defines a local coordinate system with an optional scroll and clip region.
Any node with `scroll { ... }` opens a new viewport.

Viewports are ordered within their channel, and their paint order determines visual stacking
inside that channel.

Each viewport contains one or more **layers**.

## Layers

A **layer** is a local paint-order group inside a viewport. Layers control intra-viewport stacking but 
inherit the viewport’s clip and scroll.

Viewport layers:

| Layer       | Description                                      | Scrolls with Viewport? | Notes                      |
|-------------|--------------------------------------------------|------------------------|----------------------------|
| **base**    | Main layout and content.                         | ✅                      | Default content placement. |
| **overlay** | Floating elements that scroll with the viewport. | ✅                      | Toolbars, local popups.    |

```
base < overlay
```

Layers can contain any nodes, including additional viewports (for nested scroll regions).


## Deterministic Ordering

The global paint and hit-test order is computed as:

```
(channel_rank, viewport_rank, layer_rank, local_order)
```

Where:

* `channel_rank`: global z-order (main=0, modal=1, snack=2, tooltip=3)
* `viewport_rank`: document order within the channel
* `layer_rank`: base < overlay
* `local_order`: element order within layer

Events are dispatched in reverse order of painting (topmost first).

## Conceptual Example

```text
scene {
    channel main {
        viewport {
            scroll { vertical }

            layer base {
                column { /* main content */ }
            }

            layer overlay {
                box { position { bottom: 0 right: 0 } /* floating toolbar */ }
            }
        }
    }

    channel modal {
        viewport {
            layer base {
                column {
                    text { "Dialog content" }
                    viewport { scroll { vertical } /* scrollable modal body */ }
                }
            }
        }
    }

    channel snack {
        viewport {
            layer base {
                snackbar { text { "Saved successfully" } }
            }
        }
    }

    channel tooltip {
        viewport {
            layer base {
                box { text { "Save changes" } }
            }
        }
    }
}
```