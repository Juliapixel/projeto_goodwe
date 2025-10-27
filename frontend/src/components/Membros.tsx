import type { TableHTMLAttributes } from "react";
import { twMerge } from "tailwind-merge";

const members: { name: string; rm: number }[] = [
    { name: "Allan de Souza Cardoso", rm: 561721 },
    { name: "Eduardo Bacelar Rudner", rm: 564925 },
    { name: "Giovana Dias Valentini", rm: 562390 },
    { name: "Júlia Borges Paschoalinoto", rm: 564725 },
    { name: "Raquel Amaral de Oliveira", rm: 566491 },
];

export default function Membros({
    className,
    ...rest
}: TableHTMLAttributes<HTMLTableElement>) {
    const rows = members.map(({ name, rm }) => {
        return (
            <tr className="max-w-fit" key={rm}>
                <td>{name}</td>
                <td className="pl-6">RM: {rm}</td>
            </tr>
        );
    });
    return (
        <table className={twMerge("max-w-fit mx-auto", className)} {...rest}>
            <tbody>{rows}</tbody>
        </table>
    );
}
