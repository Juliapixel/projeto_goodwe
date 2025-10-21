import { Link } from "react-router";

const members: { name: string; rm: number }[] = [
    { name: "Allan de Souza Cardoso", rm: 561721 },
    { name: "Eduardo Bacelar Rudner", rm: 564925 },
    { name: "Giovana Dias Valentini", rm: 562390 },
    { name: "Júlia Borges Paschoalinoto", rm: 564725 },
    { name: "Raquel Amaral de Oliveira", rm: 566491 },
];

export default function Footer() {
    const rows = members.map(({ name, rm }) => {
        return (
            <tr className="max-w-fit" key={rm}>
                <td>{name}</td>
                <td className="pl-6">RM: {rm}</td>
            </tr>
        );
    });
    return (
        <>
            <footer className="grid gap-6 md:gap-0 md:grid-cols-2 w-screen py-6 border-t border-shis-700 bg-shis-900  text-shis-400">
                <table className="max-w-fit mx-auto">
                    <tbody>{rows}</tbody>
                </table>
                <div className="mx-auto">
                    <p>
                        <Link className="underline" to="/">
                            Home
                        </Link>
                    </p>
                    <p>
                        <Link className="underline" to="/demo">
                            Demo
                        </Link>
                    </p>
                    <p>
                        <a
                            className="underline"
                            href="https://github.com/Juliapixel/projeto_goodwe"
                        >
                            Github
                        </a>
                    </p>
                </div>
            </footer>
            <div className="fixed w-screen h-screen bg-shis-900">&nbsp;</div>
        </>
    );
}
