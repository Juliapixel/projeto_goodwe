import { Link } from "react-router";
import GoodweLogo from "../assets/GoodWe.svg";
import GoodweW from "../assets/GoodWe_W.svg";

export default function Header() {
    return (
        <header className="fixed from-50% from-shis-950 to-transparent bg-gradient-to-b top-0 left-0 z-10 md:p-8 w-screen">
            <div className="flex shadow-md justify-between px-4 md:px-8 w-full h-18 md:h-20 md:rounded-3xl bg-zinc-200 border border-white ">
                <Link to="/" className="contents">
                    <img
                        className="not-md:hidden h-8 my-auto"
                        alt="GoodWe"
                        src={GoodweLogo}
                    />
                    <img className="md:hidden h-12 my-auto" src={GoodweW} />
                </Link>
                <div className="flex gap-6 my-auto text-black font-medium text-xl">
                    <Link to="/sobre" className="my-auto font-semibold">
                        Sobre
                    </Link>
                    <Link to="/dados" className="my-auto font-semibold">
                        Dados
                    </Link>
                    <Link
                        to="/dispositivos"
                        className="py-2 px-6 rounded-full bg-gradient-to-l to-red-400 from-red-500 text-white font-semibold"
                    >
                        Dispositivos
                    </Link>
                </div>
            </div>
        </header>
    );
}
