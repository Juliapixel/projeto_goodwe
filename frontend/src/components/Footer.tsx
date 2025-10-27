import { Link } from "react-router";
import Membros from "./Membros";

export default function Footer() {
    return (
        <>
            <footer className="grid gap-6 md:gap-0 md:grid-cols-2 w-screen py-6 border-t border-shis-700 bg-shis-900  text-shis-400">
                <Membros />
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
