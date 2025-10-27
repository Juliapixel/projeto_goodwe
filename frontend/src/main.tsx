import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import Home from "./Home.tsx";
import { createBrowserRouter, RouterProvider } from "react-router";
import Layout from "./Layout.tsx";
import Dispositivos from "./dispositivos";
import Sobre from "./sobre";
import Dados from "./dados";
import Fallback from "./Fallback.tsx";
import Demo from "./demo";

const router = createBrowserRouter([
    {
        Component: Layout,
        children: [
            {
                path: "/dispositivos",
                Component: Dispositivos,
            },
            {
                path: "/demo",
                Component: Demo,
            },
            {
                path: "/sobre",
                Component: Sobre,
            },
            {
                path: "/dados",
                Component: Dados,
            },
            {
                path: "*",
                Component: Fallback,
            },
        ],
    },
    {
        path: "/",
        Component: Home,
    },
]);

createRoot(document.getElementById("root")!).render(
    <StrictMode>
        <RouterProvider router={router} />
    </StrictMode>,
);
