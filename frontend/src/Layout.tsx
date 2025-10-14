import { Outlet } from "react-router";
import Header from "./components/Header";
import Footer from "./components/Footer";

export default function Layout() {
    return (
        <>
            <Header />
            <main className="mt-18 md:mt-36 mb-6 md:px-20 px-2 md:pt-0 pt-2">
                <Outlet />
            </main>
            <Footer />
        </>
    );
}
