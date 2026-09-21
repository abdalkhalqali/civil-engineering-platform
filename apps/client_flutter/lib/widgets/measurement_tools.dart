import 'dart:math' as math;
import 'package:flutter/material.dart';

/// Measurement data that can be displayed on the viewport.
class MeasurementResult {
  const MeasurementResult({
    required this.type,
    required this.value,
    required this.unit,
    required this.start,
    required this.end,
  });

  final MeasurementType type;
  final double value;
  final String unit;
  final Offset start;
  final Offset end;

  String get displayValue => '${value.toStringAsFixed(3)} $unit';
}

enum MeasurementType { distance, angle, area, elevation, horizontal, vertical }

/// Overlay that shows measurement results on the viewport.
class MeasurementOverlay extends StatelessWidget {
  const MeasurementOverlay({
    required this.measurements,
    super.key,
  });

  final List<MeasurementResult> measurements;

  @override
  Widget build(BuildContext context) {
    if (measurements.isEmpty) return const SizedBox.shrink();

    return Stack(
      children: [
        // Measurement lines on canvas
        for (final m in measurements)
          CustomPaint(
            size: Size.infinite,
            painter: _MeasurementPainter(measurement: m),
          ),
        // Measurement labels
        for (final m in measurements)
          Positioned(
            left: (m.start.dx + m.end.dx) / 2 - 40,
            top: (m.start.dy + m.end.dy) / 2 - 20,
            child: _MeasurementLabel(measurement: m),
          ),
      ],
    );
  }
}

class _MeasurementPainter extends CustomPainter {
  const _MeasurementPainter({required this.measurement});
  final MeasurementResult measurement;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xffef736a)
      ..strokeWidth = 1.5
      ..style = PaintingStyle.stroke;

    // Draw measurement line
    canvas.drawLine(measurement.start, measurement.end, paint);

    // Draw endpoints
    final dotPaint = Paint()
      ..color = const Color(0xffef736a)
      ..style = PaintingStyle.fill;
    canvas.drawCircle(measurement.start, 3, dotPaint);
    canvas.drawCircle(measurement.end, 3, dotPaint);

    // Draw extension lines
    final dir = measurement.end - measurement.start;
    final len = dir.distance;
    if (len > 0) {
      final nx = -dir.dy / len * 8;
      final ny = dir.dx / len * 8;
      final extPaint = Paint()
        ..color = const Color(0x80ef736a)
        ..strokeWidth = 0.7;
      canvas.drawLine(
        measurement.start,
        measurement.start + Offset(nx, ny),
        extPaint,
      );
      canvas.drawLine(
        measurement.end,
        measurement.end + Offset(nx, ny),
        extPaint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _MeasurementPainter oldDelegate) =>
      oldDelegate.measurement != measurement;
}

class _MeasurementLabel extends StatelessWidget {
  const _MeasurementLabel({required this.measurement});
  final MeasurementResult measurement;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: const Color(0xdd0b1d2c),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xffef736a).withOpacity(0.5)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            _typeIcon(measurement.type),
            size: 12,
            color: const Color(0xffef736a),
          ),
          const SizedBox(width: 4),
          Text(
            measurement.displayValue,
            style: const TextStyle(
              color: Color(0xffef736a),
              fontSize: 11,
              fontWeight: FontWeight.w600,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
    );
  }

  IconData _typeIcon(MeasurementType type) {
    switch (type) {
      case MeasurementType.distance:
        return Icons.straighten;
      case MeasurementType.angle:
        return Icons.architecture;
      case MeasurementType.area:
        return Icons.square_foot;
      case MeasurementType.elevation:
        return Icons.height;
      case MeasurementType.horizontal:
        return Icons.swap_horiz;
      case MeasurementType.vertical:
        return Icons.swap_vert;
    }
  }
}

/// Computes distance between two screen points.
double computeScreenDistance(Offset a, Offset b) {
  return (a - b).distance;
}

/// Computes angle between two screen points in degrees.
double computeScreenAngle(Offset a, Offset b) {
  return math.atan2(b.dy - a.dy, b.dx - a.dx) * 180 / math.pi;
}
