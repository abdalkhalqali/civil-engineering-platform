import 'dart:math' as math;
import 'package:flutter/material.dart';

/// Types of snap points available in the engineering environment.
enum SnapType {
  gridIntersection,
  gridLine,
  endpoint,
  midpoint,
  center,
  origin,
  perpendicular,
  parallel,
  horizontal,
  vertical,
}

/// A snap point with its position and type.
class SnapPoint {
  const SnapPoint({
    required this.position,
    required this.type,
    this.label,
  });

  final Offset position;
  final SnapType type;
  final String? label;

  Color get color {
    switch (type) {
      case SnapType.gridIntersection:
        return const Color(0xff54e0d7);
      case SnapType.gridLine:
        return const Color(0xff54e0d7).withOpacity(0.6);
      case SnapType.endpoint:
        return const Color(0xff62eee2);
      case SnapType.midpoint:
        return const Color(0xfff3c969);
      case SnapType.center:
        return const Color(0xff64d69b);
      case SnapType.origin:
        return const Color(0xffef736a);
      case SnapType.perpendicular:
        return const Color(0xff6caaff);
      case SnapType.parallel:
        return const Color(0xff6caaff).withOpacity(0.6);
      case SnapType.horizontal:
        return const Color(0xff64d69b).withOpacity(0.6);
      case SnapType.vertical:
        return const Color(0xffef736a).withOpacity(0.6);
    }
  }

  String get label_ {
    switch (type) {
      case SnapType.gridIntersection:
        return 'تقاطع شبكة';
      case SnapType.gridLine:
        return 'خط شبكة';
      case SnapType.endpoint:
        return 'نقطة نهاية';
      case SnapType.midpoint:
        return 'نقطة منتصف';
      case SnapType.center:
        return 'مركز';
      case SnapType.origin:
        return 'نقطة الأصل';
      case SnapType.perpendicular:
        return 'متعامد';
      case SnapType.parallel:
        return 'متوازي';      case SnapType.horizontal:
        return 'أفقي';
      case SnapType.vertical:
        return 'رأسي';
    }
  }
}

/// Constraint indicator showing during drawing.
class ConstraintIndicator {
  const ConstraintIndicator({
    required this.type,
    required this.line,
  });

  final ConstraintType type;
  final (Offset, Offset) line;
}

enum ConstraintType { horizontal, vertical, parallel, perpendicular, equal }

/// Overlay that renders snap points and constraint indicators.
class SnapIndicator extends StatelessWidget {
  const SnapIndicator({
    required this.activeSnap,
    required this.constraints,
    required this.showSnap,
    super.key,
  });

  final SnapPoint? activeSnap;
  final List<ConstraintIndicator> constraints;
  final bool showSnap;

  @override
  Widget build(BuildContext context) {
    if (!showSnap) return const SizedBox.shrink();

    return CustomPaint(
      size: Size.infinite,
      painter: _SnapPainter(
        activeSnap: activeSnap,
        constraints: constraints,
      ),
    );
  }
}

class _SnapPainter extends CustomPainter {
  const _SnapPainter({
    required this.activeSnap,
    required this.constraints,
  });

  final SnapPoint? activeSnap;
  final List<ConstraintIndicator> constraints;

  @override
  void paint(Canvas canvas, Size size) {
    // Draw constraint lines
    for (final constraint in constraints) {
      final paint = Paint()
        ..color = _constraintColor(constraint.type)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke
        ..strokeCap = StrokeCap.round;

      // Dashed line
      _drawDashedLine(canvas, constraint.line.$1, constraint.line.$2, paint);
    }

    // Draw active snap point
    if (activeSnap != null) {
      final snap = activeSnap!;

      // Crosshair
      final crosshairPaint = Paint()
        ..color = snap.color
        ..strokeWidth = 1.0;

      const crossSize = 12.0;
      canvas.drawLine(
        snap.position + const Offset(-crossSize, 0),
        snap.position + const Offset(crossSize, 0),
        crosshairPaint,
      );
      canvas.drawLine(
        snap.position + const Offset(0, -crossSize),
        snap.position + const Offset(0, crossSize),
        crosshairPaint,
      );

      // Diamond marker
      final diamondPaint = Paint()
        ..color = snap.color
        ..style = PaintingStyle.fill;
      final path = Path()
        ..moveTo(snap.position.dx, snap.position.dy - 5)
        ..lineTo(snap.position.dx + 5, snap.position.dy)
        ..lineTo(snap.position.dx, snap.position.dy + 5)
        ..lineTo(snap.position.dx - 5, snap.position.dy)
        ..close();
      canvas.drawPath(path, diamondPaint);

      // Snap label
      if (snap.label != null) {
        final textPainter = TextPainter(
          text: TextSpan(
            text: snap.label_,
            style: TextStyle(
              color: snap.color,
              fontSize: 10,
              fontWeight: FontWeight.w600,
            ),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        textPainter.paint(
          canvas,
          snap.position + const Offset(10, -14),
        );
      }
    }
  }

  Color _constraintColor(ConstraintType type) {
    switch (type) {
      case ConstraintType.horizontal:
        return const Color(0xff64d69b);
      case ConstraintType.vertical:
        return const Color(0xffef736a);
      case ConstraintType.parallel:
        return const Color(0xff6caaff);
      case ConstraintType.perpendicular:
        return const Color(0xff6caaff);
      case ConstraintType.equal:
        return const Color(0xfff3c969);
    }
  }

  void _drawDashedLine(Canvas canvas, Offset start, Offset end, Paint paint) {
    final dx = end.dx - start.dx;
    final dy = end.dy - start.dy;
    final len = math.sqrt(dx * dx + dy * dy);
    if (len == 0) return;

    const dashLength = 6.0;
    const gapLength = 4.0;
    final steps = len / (dashLength + gapLength);
    final dirX = dx / len;
    final dirY = dy / len;

    for (var i = 0; i < steps; i++) {
      final startOffset = i * (dashLength + gapLength);
      final dashStart = Offset(
        start.dx + dirX * startOffset,
        start.dy + dirY * startOffset,
      );
      final dashEnd = Offset(
        start.dx + dirX * (startOffset + dashLength),
        start.dy + dirY * (startOffset + dashLength),
      );
      canvas.drawLine(dashStart, dashEnd, paint);
    }
  }

  @override
  bool shouldRepaint(covariant _SnapPainter oldDelegate) =>
      oldDelegate.activeSnap != activeSnap ||
      oldDelegate.constraints.length != constraints.length;
}

/// Smart constraints engine that detects drawing constraints.
class SmartConstraints {
  /// Detects constraints between the current point and reference points.
  static List<ConstraintIndicator> detect({
    required Offset current,
    required Offset previous,
    double tolerance = 0.1,
  }) {
    final constraints = <ConstraintIndicator>[];

    final dx = (current.dx - previous.dx).abs();
    final dy = (current.dy - previous.dy).abs();

    // Horizontal constraint
    if (dy < tolerance && dx > tolerance) {
      constraints.add(ConstraintIndicator(
        type: ConstraintType.horizontal,
        line: (previous, Offset(current.dx, previous.dy)),
      ));
    }

    // Vertical constraint
    if (dx < tolerance && dy > tolerance) {
      constraints.add(ConstraintIndicator(
        type: ConstraintType.vertical,
        line: (previous, Offset(previous.dx, current.dy)),
      ));
    }

    return constraints;
  }
}
