import { useState } from "react";
import {
    CartesianGrid,
    Legend,
    Line,
    LineChart,
    ReferenceArea,
    ReferenceLine,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
    type TooltipContentProps,
} from "recharts";
import type { CategoricalChartFunc } from "recharts/types/chart/types";

function tooltipRender(props: TooltipContentProps<number, string>) {
    const values = props.payload.map((p) => {
        return (
            <div>
                <span style={{ color: p.color }}>{p.name}</span>{" "}
                <span>{Math.round(p.value * 100) / 100}</span>
            </div>
        );
    });
    return (
        <div className="bg-zinc-800/40 border border-zinc-300/10 rounded-lg p-2 backdrop-blur-sm">
            <p>
                {props.payload[0]?.payload.econ
                    ? "Economia ativada "
                    : "Economia desativada "}
            </p>
            <p>{props.label}</p>
            {values}
        </div>
    );
}

interface ChartProps {
    onClickEcon?(econ: boolean): void;
}

export default function LoadChart({ onClickEcon }: ChartProps) {
    // TODO: pegar data do backend
    const data = Array.from({ length: 12 * 24 }, (_, i) => {
        const time = new Date(2025, 10, 9, Math.floor(i / 12), (i % 12) * 5, 0);
        const timeStr = time.toLocaleTimeString();
        const value =
            (Math.sin(i / 24) + 1) * 500 +
            (Math.sin(i / 5) + 1) * 400 +
            (Math.sin(i / 4) + 1) * 200;
        const econ = value < 400;
        return {
            time: timeStr,
            value: value,
            econ: econ,
        };
    });

    const runs: { x1: string; x2?: string }[] = [];
    for (const i of data) {
        if (i.econ) {
            if (runs.length == 0 || runs[runs.length - 1]?.x2) {
                runs.push({ x1: i.time });
            }
        } else {
            if (runs.length > 0 && !runs[runs.length - 1].x2) {
                runs[runs.length - 1].x2 = i.time;
            }
        }
    }
    if (runs.length > 0 && runs[runs.length - 1].x2 === undefined) {
        runs[runs.length - 1].x2 = data[data.length - 1].time;
    }

    const areas = runs.map((a, i) => (
        <ReferenceArea
            key={i}
            stroke="green"
            fill="green"
            fillOpacity={0.2}
            x1={a.x1}
            x2={a.x2}
        />
    ));

    const [lineX, setLineX] = useState<string | undefined>();

    const timeSelect: CategoricalChartFunc = (e) => {
        setLineX(e.activeLabel);
        const isOnEcon =
            data.find((d) => d.time == e.activeLabel)?.econ ?? false;
        onClickEcon?.(isOnEcon);
    };

    return (
        <ResponsiveContainer>
            <LineChart data={data} onMouseDown={timeSelect}>
                <CartesianGrid stroke="#ffffff30" strokeDasharray={"3 3"} />
                {areas}
                <Line
                    isAnimationActive={false}
                    dataKey={"value"}
                    name="Consumo (W)"
                    stroke="#366fe8"
                    dot={false}
                    strokeWidth={2}
                />
                <XAxis dataKey={"time"} stroke="#ffffff90" />
                <YAxis stroke="#ffffff90" />
                <Legend align="right" verticalAlign="top" />
                <Tooltip animationDuration={100} content={tooltipRender} />
                <ReferenceLine x={lineX} strokeWidth={2} />
            </LineChart>
        </ResponsiveContainer>
    );
}
