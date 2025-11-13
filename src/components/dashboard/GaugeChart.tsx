import { Card } from '@/components/ui/card';
import {
  PieChart as RechartsPieChart,
  Pie,
  Cell,
  ResponsiveContainer,
} from 'recharts';

export interface GaugeChartProps {
  title: string;
  value: number;
  max?: number;
  min?: number;
  unit?: string;
  thresholds?: {
    low: number;
    medium: number;
    high: number;
  };
  colors?: {
    low: string;
    medium: string;
    high: string;
    background: string;
  };
  height?: number;
  className?: string;
}

const DEFAULT_COLORS = {
  low: '#10b981', // green
  medium: '#f59e0b', // yellow
  high: '#ef4444', // red
  background: '#e5e7eb', // gray
};

export function GaugeChart({
  title,
  value,
  max = 100,
  min = 0,
  unit = '%',
  thresholds = { low: 33, medium: 66, high: 100 },
  colors = DEFAULT_COLORS,
  height = 200,
  className = '',
}: GaugeChartProps) {
  const normalizedValue = ((value - min) / (max - min)) * 100;

  const getColor = (val: number): string => {
    if (val <= thresholds.low) return colors.low;
    if (val <= thresholds.medium) return colors.medium;
    return colors.high;
  };

  const currentColor = getColor(normalizedValue);

  const data = [
    { name: 'Value', value: normalizedValue },
    { name: 'Remaining', value: 100 - normalizedValue },
  ];

  return (
    <Card className={`p-6 ${className}`}>
      <h3 className="text-lg font-semibold mb-2">{title}</h3>
      <div className="flex flex-col items-center">
        <ResponsiveContainer width="100%" height={height}>
          <RechartsPieChart>
            <Pie
              data={data}
              cx="50%"
              cy="50%"
              startAngle={180}
              endAngle={0}
              innerRadius="70%"
              outerRadius="90%"
              paddingAngle={0}
              dataKey="value"
            >
              <Cell fill={currentColor} />
              <Cell fill={colors.background} />
            </Pie>
          </RechartsPieChart>
        </ResponsiveContainer>
        <div className="mt-2 text-center">
          <p className="text-3xl font-bold" style={{ color: currentColor }}>
            {value.toFixed(1)}
            {unit}
          </p>
          <p className="text-sm text-muted-foreground mt-1">
            {min} - {max}
            {unit} 범위
          </p>
        </div>
      </div>
    </Card>
  );
}
