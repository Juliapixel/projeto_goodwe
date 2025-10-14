import { Cell, Label, Pie, PieChart, ResponsiveContainer } from "recharts";

export default function Battery({ charge }: { charge: number }) {
    const data = [
        {value: charge, name: "Carga (%)"},
        {value: 100-charge, name: ""}
    ]
    let col = "lime";
    if (charge < 20) {
        col = "red"
    } else if (charge < 50) {
        col = "yellow"
    }
    return (
        <>
            <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                    <Pie data={data} innerRadius={"75%"} outerRadius={"100%"} startAngle={90} endAngle={-270} stroke="none" >
                        <Cell fill={col} />
                        <Cell fill="none" />
                        <Label className="select-none" textAnchor="middle" position={"center"} fill="white" fontSize="200%" fontWeight={600} >{`${charge}%`}</Label>
                    </Pie>
                </PieChart>
            </ResponsiveContainer>
        </>
    )
}
