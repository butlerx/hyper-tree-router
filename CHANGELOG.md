# Changelog

## [0.2.0](https://github.com/butlerx/hyper-tree-router/compare/v0.1.0...v0.2.0) (2026-11-16)


### ⚠ BREAKING CHANGES

* capture keys drop the `:` prefix, so `captures.get(":id")` becomes `get("id")`. Router::new is now crate-private; build routers with RouterBuilder.
* handlers must return a future. Wrap existing sync handlers as `async fn`. Handler is no longer a plain fn pointer.
* requires hyper 1.x. Route, RouterBuilder and Router are generic over body types, RouterSvc is removed and Router.routes is private.

### Features

* catch-all routes, HEAD fallback and bare param names ([#28](https://github.com/butlerx/hyper-tree-router/issues/28)) ([7b7e83b](https://github.com/butlerx/hyper-tree-router/commit/7b7e83ba0c6ef5709dcd2d4282f4117efc00de7e))
* custom error handler and Allow header on 405 ([#27](https://github.com/butlerx/hyper-tree-router/issues/27)) ([422353d](https://github.com/butlerx/hyper-tree-router/commit/422353df65e1ed3cadeab96b84f6cb677baae094))
* migrate to hyper 1.x and implement tower Service ([#24](https://github.com/butlerx/hyper-tree-router/issues/24)) ([54964f3](https://github.com/butlerx/hyper-tree-router/commit/54964f39aa978a70b2bdf68b524ea24a6d4d1732))
* support async handlers and closures that capture state ([#25](https://github.com/butlerx/hyper-tree-router/issues/25)) ([5d82a12](https://github.com/butlerx/hyper-tree-router/commit/5d82a12cde8c8c172fb6b95281a344f2169194a6))
* typed URL params and an example for every feature ([#29](https://github.com/butlerx/hyper-tree-router/issues/29)) ([25e39d1](https://github.com/butlerx/hyper-tree-router/commit/25e39d1d6ff14d5ffb6e2b539e997f058aa48cdd))
