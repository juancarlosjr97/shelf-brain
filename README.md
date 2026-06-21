# ShelfBrain

ShelfBrain is an attention-based personal organization app.

Instead of forcing every thought into projects, folders, or task lists, ShelfBrain asks one simple question:

> How much attention does this deserve?

Users capture items onto one of three shelves:

- **Top Shelf**: needs attention now or soon
- **Middle Shelf**: important, but not urgent
- **Bottom Shelf**: ideas, aspirations, and future possibilities

## MVP Direction

The first version is a mobile app built with **React Native + Expo**. The backend will likely use **Supabase** for data, sync, sharing, and storage. Authentication can start with Supabase Auth for speed, with **Auth0** reserved for a later product stage if the authentication requirements become more advanced.

Rust is valuable for future background activity, reminder scheduling, local indexing, sync workers, and platform-specific performance work, but the MVP should avoid introducing native complexity before the capture and review loop is proven.

## Docs

- [MVP Product Spec](docs/MVP_PRODUCT_SPEC.md)
- [Technical Architecture](docs/TECHNICAL_ARCHITECTURE.md)
- [Lean Design Brief](docs/LEAN_DESIGN_BRIEF.md)

## One Sentence Pitch

ShelfBrain is a priority-based second brain that helps people organize thoughts, ideas, reminders, and tasks according to how much attention they deserve rather than forcing everything into projects and to-do lists.
