import {
    Bar,
    BarChart,
    CartesianGrid,
    Legend,
    Line,
    LineChart,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
    type TooltipContentProps,
} from "recharts";

function tooltipRender(props: TooltipContentProps<number, string>) {
    console.log(props);
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
            <p>{props.label}</p>
            {values}
        </div>
    );
}

function MultiChart() {
    const data: { time: string; value: number; two: number }[] = new Array(
        12 * 24,
    );
    for (let i = 0; i < data.length; i++) {
        const time = new Date(2025, 10, 9, Math.floor(i / 12), (i % 12) * 5, 0);
        const timeStr = time.toLocaleTimeString();
        data[i] = {
            time: timeStr,
            value: Math.sin(i / 5) * 0.5 + Math.sin(i / 20),
            two: Math.sin(i / 5) * 0.1 - Math.sin(i / 50),
        };
        console.log(data[i]);
    }
    return (
        <ResponsiveContainer>
            <LineChart data={data}>
                <CartesianGrid stroke="#ffffff30" strokeDasharray={"3 3"} />
                <Line
                    dataKey={"value"}
                    name="Consumo (W)"
                    stroke="lime"
                    dot={false}
                    strokeWidth={2}
                />
                <Line
                    dataKey={"two"}
                    name="Geração (W)"
                    stroke="orange"
                    dot={false}
                    strokeWidth={2}
                />
                <XAxis dataKey={"time"} stroke="#ffffff90" />
                <YAxis stroke="#ffffff90" />
                <Legend align="right" />
                <Tooltip animationDuration={100} content={tooltipRender} />
            </LineChart>
        </ResponsiveContainer>
    );
}

function Economia() {
    const DAYS = [
        "Segunda-feira",
        "Terça-feira",
        "Quarta-feira",
        "Quinta-feira",
        "Sexta-feira",
        "Sábado",
        "Domingo",
    ];
    const data = Array(7)
        .fill(undefined)
        .map((_, i) => {
            return {
                day: DAYS[i],
                value: Math.random(),
            };
        });
    return (
        <ResponsiveContainer>
            <BarChart data={data}>
                <CartesianGrid stroke="#ffffff30" strokeDasharray={"3 3"} />
                <XAxis dataKey={"day"} stroke="#ffffff90" />
                <YAxis stroke="#ffffff90" />
                <Legend align="right" />
                <Tooltip animationDuration={100} content={tooltipRender} />
                <Bar dataKey={"value"} name={"Economia (kWh)"} fill="red" />
            </BarChart>
        </ResponsiveContainer>
    );
}

export default function Home() {
    return (
        <div className="">
            <div className="mb-10">
                <h1 className="text-3xl font-bold">
                    Solar House Integrated Systems
                </h1>
                <p>A sua central de controle de produtos em stand-by!</p>
            </div>
            <div className="grid md:grid-cols-2 w-full md:h-72 h-96 md:px-8">
                <MultiChart />
                <Economia />
            </div>
        </div>
    );
}
