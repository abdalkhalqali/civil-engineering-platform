import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Floating overlay that appears near the cursor during drawing/editing,
/// showing real-time measurements and allowing numeric input.
///
/// Follows the principle: press → drag → see measurement → edit number → release → element created.
class DynamicInputOverlay extends StatefulWidget {
  const DynamicInputOverlay({
    required this.visible,
    required this.measurements,
    this.onMeasurementChanged,
    this.onSubmit,
    this.position,
    super.key,
  });

  final bool visible;
  final List<MeasurementField> measurements;
  final ValueChanged<Map<String, double>>? onMeasurementChanged;
  final VoidCallback? onSubmit;
  final Offset? position;

  @override
  State<DynamicInputOverlay> createState() => _DynamicInputOverlayState();
}

class MeasurementField {
  const MeasurementField({
    required this.key,
    required this.label,
    required this.value,
    this.unit = 'm',
    this.editable = true,
  });

  final String key;
  final String label;
  final double value;
  final String unit;
  final bool editable;

  MeasurementField copyWith({double? value}) {
    return MeasurementField(
      key: key,
      label: label,
      value: value ?? this.value,
      unit: unit,
      editable: editable,
    );
  }
}

class _DynamicInputOverlayState extends State<DynamicInputOverlay>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _opacity;
  final Map<String, TextEditingController> _controllers = {};
  String? _focusedField;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 150),
      vsync: this,
    );
    _opacity = CurvedAnimation(parent: _controller, curve: Curves.easeOut);
    if (widget.visible) _controller.forward();
  }

  @override
  void didUpdateWidget(covariant DynamicInputOverlay oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.visible && !oldWidget.visible) {
      _controller.forward(from: 0);
    } else if (!widget.visible && oldWidget.visible) {
      _controller.reverse();
    }
    // Update controllers with new values
    for (final m in widget.measurements) {
      _controllers.putIfAbsent(m.key, () => TextEditingController());
      if (_focusedField != m.key) {
        _controllers[m.key]!.text = m.value.toStringAsFixed(3);
      }
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    for (final c in _controllers.values) {
      c.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.visible) return const SizedBox.shrink();

    return FadeTransition(
      opacity: _opacity,
      child: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: const Color(0xdd0b1d2c),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: const Color(0xff2a5260)),
          boxShadow: const [
            BoxShadow(
              color: Color(0x40000000),
              blurRadius: 8,
              offset: Offset(0, 2),
            ),
          ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const _Header(label: 'الإدخال'),
            const SizedBox(height: 6),
            for (final m in widget.measurements) ...[
              _MeasurementRow(
                field: m,
                controller: _controllers.putIfAbsent(
                  m.key,
                  () => TextEditingController(
                    text: m.value.toStringAsFixed(3),
                  ),
                ),
                isFocused: _focusedField == m.key,
                onFocusChanged: (focused) {
                  setState(() {
                    _focusedField = focused ? m.key : null;
                  });
                },
                onChanged: (text) {
                  final value = double.tryParse(text);
                  if (value != null && widget.onMeasurementChanged != null) {
                    final values = <String, double>{};
                    for (final entry in _controllers.entries) {
                      final v = double.tryParse(entry.value.text);
                      if (v != null) values[entry.key] = v;
                    }
                    widget.onMeasurementChanged!(values);
                  }
                },
                onSubmitted: widget.onSubmit,
              ),
              if (m != widget.measurements.last) const SizedBox(height: 4),
            ],
          ],
        ),
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.label});
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(Icons.straighten, color: Color(0xff54e0d7), size: 14),
        const SizedBox(width: 6),
        Text(
          label,
          style: const TextStyle(
            color: Color(0xffd5e1e7),
            fontSize: 11,
            fontWeight: FontWeight.w700,
          ),
        ),
      ],
    );
  }
}

class _MeasurementRow extends StatefulWidget {
  const _MeasurementRow({
    required this.field,
    required this.controller,
    required this.isFocused,
    required this.onFocusChanged,
    required this.onChanged,
    required this.onSubmitted,
  });

  final MeasurementField field;
  final TextEditingController controller;
  final bool isFocused;
  final ValueChanged<bool> onFocusChanged;
  final ValueChanged<String> onChanged;
  final VoidCallback? onSubmitted;

  @override
  State<_MeasurementRow> createState() => _MeasurementRowState();
}

class _MeasurementRowState extends State<_MeasurementRow> {
  final FocusNode _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _focusNode.addListener(() {
      widget.onFocusChanged(_focusNode.hasFocus);
    });
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: 40,
          child: Text(
            widget.field.label,
            style: const TextStyle(
              color: Color(0xff7793a3),
              fontSize: 10,
            ),
          ),
        ),
        const SizedBox(width: 4),
        SizedBox(
          width: 72,
          height: 26,
          child: TextField(
            controller: widget.controller,
            focusNode: _focusNode,
            keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true),
            inputFormatters: [
              FilteringTextInputFormatter.allow(RegExp(r'^-?\d*\.?\d*')),
            ],
            style: const TextStyle(
              color: Color(0xff62eee2),
              fontSize: 11,
              fontFamily: 'monospace',
            ),
            textAlign: TextAlign.center,
            decoration: InputDecoration(
              contentPadding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(4),
                borderSide: BorderSide(
                  color: widget.isFocused
                      ? const Color(0xff54e0d7)
                      : const Color(0xff2a5260),
                ),
              ),
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(4),
                borderSide: const BorderSide(color: Color(0xff2a5260)),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(4),
                borderSide: const BorderSide(color: Color(0xff54e0d7), width: 1.5),
              ),
              filled: true,
              fillColor: const Color(0xff081928),
              isDense: true,
            ),
            onChanged: widget.onChanged,
            onSubmitted: (_) => widget.onSubmitted?.call(),
          ),
        ),
        const SizedBox(width: 3),
        Text(
          widget.field.unit,
          style: const TextStyle(
            color: Color(0xff54e0d7),
            fontSize: 9,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    );
  }
}
