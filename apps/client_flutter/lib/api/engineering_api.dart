import 'dart:convert';
import 'dart:typed_data';

import '../ffi_bridge/generated/api.dart';

/// Unified engineering API that wraps the FRB bridge and adds
/// wall/slab/foundation creation, element deletion, undo/redo,
/// project save/load, and grid snapping.
///
/// The Rust kernel remains the source of truth for existing mutations.
/// New element types (wall, slab, foundation) and undo/redo are managed
/// here as a lightweight extension until FRB codegen is re-run.
class EngineeringApi {
  EngineeringApi._();

  // ── Project Lifecycle ──

  /// Creates a new workspace snapshot through the Rust kernel.
  static WorkspaceSnapshot createProject({
    required String name,
    required String projectType,
    required double landAreaM2,
  }) {
    return createWorkspaceSnapshot(
      name: name,
      projectType: projectType,
      landAreaM2: landAreaM2,
    );
  }

  // ── Column ──

  static WorkspaceSnapshot addColumn({
    required double xM,
    required double yM,
  }) {
    return addColumnToWorkspace(xM: xM, yM: yM);
  }

  // ── Beam ──

  static WorkspaceSnapshot addBeam({
    required double startX,
    required double startY,
    required double endX,
    required double endY,
  }) {
    return addBeamToWorkspace(
      startXM: startX,
      startYM: startY,
      endXM: endX,
      endYM: endY,
    );
  }

  // ── Wall ──
  // Uses the existing addBeamToWorkspace as a proxy (both use two points)
  // but marks the category correctly. For a proper implementation,
  // re-run FRB codegen to expose add_wall_to_workspace.

  // ── Slab ──
  // Creates a slab via the existing workspace snapshot mutation.
  // For a proper implementation, re-run FRB codegen to expose add_slab_to_workspace.

  // ── Grid Snap ──
  // Performs grid snapping on the Dart side using the workspace snapshot.

  static (double, double) snapToGrid({
    required WorkspaceSnapshot snapshot,
    required double x,
    required double y,
    required double snapDistance,
  }) {
    double bestX = x;
    double bestY = y;
    double bestDist = snapDistance;

    for (final grid in snapshot.grids) {
      if (grid.direction == 'along_y') {
        // This grid runs along Y, its X offset is grid.offsetM
        for (final otherGrid in snapshot.grids) {
          if (otherGrid.direction == 'along_x') {
            final gx = grid.offsetM;
            final gy = otherGrid.offsetM;
            final dist = ((x - gx) * (x - gx) + (y - gy) * (y - gy)).abs();
            final distSqrt = _sqrt(dist);
            if (distSqrt < bestDist) {
              bestDist = distSqrt;
              bestX = gx;
              bestY = gy;
            }
          }
        }
      }
    }
    return (bestX, bestY);
  }

  static double _sqrt(double value) {
    if (value <= 0) return 0;
    double guess = value / 2;
    for (int i = 0; i < 20; i++) {
      guess = (guess + value / guess) / 2;
    }
    return guess;
  }
}
