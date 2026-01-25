import React, { useEffect, useRef } from 'react';
import type { PlotData } from '../types';

interface FunctionPlotProps {
  data: PlotData;
  width?: number;
  height?: number;
  className?: string;
}

/**
 * Renders a simple function plot using Canvas.
 * Lightweight alternative to heavy charting libraries.
 */
export const FunctionPlot: React.FC<FunctionPlotProps> = ({
  data,
  width = 400,
  height = 250,
  className = '',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !data.x_values.length) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.fillStyle = '#1e1e1e';
    ctx.fillRect(0, 0, width, height);

    // Calculate bounds
    const xMin = Math.min(...data.x_values);
    const xMax = Math.max(...data.x_values);
    const yMin = Math.min(...data.y_values.filter(y => isFinite(y)));
    const yMax = Math.max(...data.y_values.filter(y => isFinite(y)));

    // Add padding
    const padding = 40;
    const plotWidth = width - 2 * padding;
    const plotHeight = height - 2 * padding;

    // Scale functions
    const scaleX = (x: number) => padding + ((x - xMin) / (xMax - xMin)) * plotWidth;
    const scaleY = (y: number) => height - padding - ((y - yMin) / (yMax - yMin)) * plotHeight;

    // Draw axes
    ctx.strokeStyle = '#555';
    ctx.lineWidth = 1;

    // X-axis
    const y0 = scaleY(0);
    if (y0 >= padding && y0 <= height - padding) {
      ctx.beginPath();
      ctx.moveTo(padding, y0);
      ctx.lineTo(width - padding, y0);
      ctx.stroke();
    }

    // Y-axis
    const x0 = scaleX(0);
    if (x0 >= padding && x0 <= width - padding) {
      ctx.beginPath();
      ctx.moveTo(x0, padding);
      ctx.lineTo(x0, height - padding);
      ctx.stroke();
    }

    // Draw grid
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 0.5;

    // Vertical grid lines
    const xStep = (xMax - xMin) / 10;
    for (let x = Math.ceil(xMin / xStep) * xStep; x <= xMax; x += xStep) {
      const sx = scaleX(x);
      ctx.beginPath();
      ctx.moveTo(sx, padding);
      ctx.lineTo(sx, height - padding);
      ctx.stroke();
    }

    // Horizontal grid lines
    const yStep = (yMax - yMin) / 8;
    for (let y = Math.ceil(yMin / yStep) * yStep; y <= yMax; y += yStep) {
      const sy = scaleY(y);
      ctx.beginPath();
      ctx.moveTo(padding, sy);
      ctx.lineTo(width - padding, sy);
      ctx.stroke();
    }

    // Draw the function
    ctx.strokeStyle = '#4fc3f7';
    ctx.lineWidth = 2;
    ctx.beginPath();

    let started = false;
    for (let i = 0; i < data.x_values.length; i++) {
      const x = data.x_values[i];
      const y = data.y_values[i];

      if (!isFinite(y)) {
        started = false;
        continue;
      }

      const sx = scaleX(x);
      const sy = scaleY(y);

      if (!started) {
        ctx.moveTo(sx, sy);
        started = true;
      } else {
        ctx.lineTo(sx, sy);
      }
    }
    ctx.stroke();

    // Draw labels
    ctx.fillStyle = '#888';
    ctx.font = '12px monospace';

    // X-axis label
    ctx.textAlign = 'center';
    ctx.fillText(data.x_label, width / 2, height - 5);

    // Y-axis label
    ctx.save();
    ctx.translate(12, height / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText(data.y_label, 0, 0);
    ctx.restore();

    // Title/label
    ctx.fillStyle = '#aaa';
    ctx.font = '14px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(data.label, width / 2, 20);

    // Axis values
    ctx.fillStyle = '#666';
    ctx.font = '10px monospace';
    ctx.textAlign = 'right';
    ctx.fillText(yMax.toFixed(2), padding - 5, padding + 10);
    ctx.fillText(yMin.toFixed(2), padding - 5, height - padding);

    ctx.textAlign = 'center';
    ctx.fillText(xMin.toFixed(1), padding, height - padding + 15);
    ctx.fillText(xMax.toFixed(1), width - padding, height - padding + 15);

  }, [data, width, height]);

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      className={`function-plot ${className}`}
      style={{ borderRadius: '8px' }}
    />
  );
};

export default FunctionPlot;
