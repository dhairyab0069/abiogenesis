# Stage 6: Animals, from sea to land

Stage 6 continues from the colonies of stage 5 and follows an animal lineage out of the sea. Like every stage it is a small,
honest simplification, not a biology textbook: every number is made up to tell a qualitative story, and none is a measurement.
(The scenario lab, formerly planned as stage 6, is now stage 7 and is not built yet.)

Open it from the launcher (*Animals: from sea to land*) or with `abiogenesis animals`.

## 1. The model (`src/animals/`)

**A body is a body plan and eleven dials.** `Traits` holds whether the body is radial (round, no front) or bilateral (a head
and a tail), and a level from 0 to 1 for each organ: mouth and gut, nerve cord and brain, eyes, muscle, fins, backbone, gills,
jaws, lungs, limbs and a sealed skin. A radial body can only grow a blind gut and a nerve net. Other organs can only grow
once the parts they come from exist: eyes need a nerve, jaws need gills and a skeleton, lungs bud off the gut, limbs grow out
of fins that have a skeleton. A child is its parent's body with a few random changes (12% per organ, a few hundredths in size);
the ceiling limits gaining a level, never losing one.

**The shore is four zones**: open sea, shallows, tidal flats and land. Each zone differs qualitatively in oxygen (air holds
more than water, and the warm shallows hold less than the sea), support (water holds a body up), drying out, food (richest in
the shallows) and predators (big hunters stay in open water). Three dials change the world: oxygen in the sea, shore steepness
(a steep shore leaves little room for shallows and flats) and predator pressure.

**Fitness is a product of six trade-offs** (food, oxygen, support, drying, predation, upkeep), so no organ is free and none is
useless everywhere. Fins and gills win in water; a lung-like organ gives an edge in stale shallows, more so the less oxygen the
water holds; limb-like fins help on the bottom and the flats; on land limbs, a skeleton, a sealed skin and lungs win. A sealed
skin costs gas exchange, which is why it only pays out of the water.

**A run is a pure function of (environment, seed).** About 200 individual animals, each with a body and a zone. Each generation
every animal has offspring in proportion to its fitness (sharpened a little) and how crowded its zone is; a few offspring
wander into the next zone and have to do well with the body they have. Nothing steers the lineage toward land: if the shore
makes land a bad place, it stays in the sea, and a world can be so hard that the lineage dies out (the run says when).
The whole run (1,600 generations) is computed up front, so the scrubber can go backwards and a what-if can be compared with the
default world.

**Milestones** are detected on the *frontier*, the most landward zone holding at least a handful of animals: symmetry and a gut,
eyes and a nerve cord, a backbone, jaws, limb-like fins, lungs, leaving the water, first land animals. Each is reached the first
time its condition holds. The simulation does not have to reach them in the story's order, and the story says so.

## 2. The picture (`src/animalviz/`, `src/app/animal_draw.rs`)

The geometry is egui-free and tested, like the cell drawings. Organs are drawn only once their level passes a small threshold,
fade in, then grow with their level. The backbone is a flexible rod (notochord) and later vertebrae and a skull. A fin and a
limb are one thing at two ends: a webbed flap that stiffens, grows an elbow and a wrist, loses its web and ends in toes.
The shoreline has a tide that floods and drains the flats; animals are placed in their zone, swim in water and prop themselves
up or stand on the ground. Radial animals are drawn round with arms, a blind gut and a nerve ring.

## 3. Side by side with the real story (`src/animals/story.rs`)

Left: the simulated lineage (scene, portrait of the leading animals with a label on every organ that has appeared, organ
levels). Right: nine short chapters (a prologue from stage 5, then one per milestone) each with a setting, a problem, an
adaptation and a result, plus a note on what the simulation did the same or differently, including whether this run reached
the milestone in the story's order. The chapter for the latest milestone is highlighted as the run plays or is scrubbed;
earlier chapters collapse to their result and any chapter can be opened by hand. The text is written, qualitative and labelled
a story, not data: times are given only with "roughly" (a test enforces that, and that chapters carry no numbers).

## 4. What if? (hypothetical)

A window with the three dials, *Try this world*, *Back to the default world* and *New seed*. It reruns the same lineage in the
new world and compares it with the default world (same seed): milestones reached, where the leading animals live, animals on
land, how many generations later or sooner the first land animals came, and the leading animals' organ levels side by side.
It is labelled hypothetical, and as one run each, where chance matters. It is the hook the stage 7 scenario lab builds on.

## Tests

`src/animals/tests.rs` covers the trait ceilings and mutation, encoding, the zones, every fitness trade-off (fins and gills in
water, lungs in stale shallows, limbs, skin and lungs on land, predators), determinism of a run, zone room, extinction
bookkeeping, milestones and their order, the story's structure and honesty, and the CSV export. `src/animalviz/tests.rs` covers
organ visibility and placement, the fin to limb change, the shore, the tide and animal placement. `src/app/animals_view.rs` has a
headless draw test in both layouts.
