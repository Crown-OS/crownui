# capture

Captures CrownUI states as editable design scenes. An app is mounted headlessly
(`crowntest::TestApp<CaptureBackend>`), rendered offscreen by Impeller with real Inter shaping,
and each frame is rebuilt from the composition: one `SceneNode` per view layer (named by its
semantic label or element path), each boundary's draw commands as rectangles, paths, live text
and images, with the box fill, outline, shadows, clip and backdrop blur hoisted onto the node.

```rust,ignore
let mut app = capture::mount(dialog(), 480.0, 320.0)?;
let mut carrier = capture::Carrier::new("crownos-polkit-agent", "Authentication dialog");
carrier.capture_state(&mut app, "Dialog", "Request", "A password prompt for one user")?;
app.click(role(Role::Button).name("Cancel"));
carrier.capture_state(&mut app, "Dialog", "Cancelled", "")?;
carrier.write(None)?; // $CROWN_DESIGN_DIR/<app>.<theme>.figma.png + renders
```

`<app>.figma.png` is a 1×1 PNG whose private chunks carry the builder (`crJs`, from
`figma/builder.js`), the page info (`crPg`), one scene per state (`crSc`, ASCII JSON) and every
image (`crIm`, 8-byte key + PNG). Upload it with the Figma MCP `upload_assets`, then run the stub
in `figma/stub.js` through `use_figma` with the returned `imageHash`, the placed node id and the
target page id. The builder lays out a page section per group with light and dark rows, and
replaces states of the same name on re-capture.

Non-goals: animation timelines, hover effects that only exist under a real pointer, compositor
chrome. Scaled layers keep their rotation but bake scale into the box size.
