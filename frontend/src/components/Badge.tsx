import type { HTMLAttributes } from "react";
import { twMerge } from "tailwind-merge";

export interface BadgeProps extends HTMLAttributes<HTMLDivElement> {
    text: string;
    dotColor: string;
}

export default function Badge({
    dotColor,
    text,
    className,
    ...rest
}: BadgeProps) {
    const cls = twMerge(
        "flex flex-row w-fit h-10 pr-4 border border-shis-600 bg-shis-900 rounded-full",
        className,
    );
    return (
        <div className={cls} {...rest}>
            <svg className="p-2" viewBox="-0.5 -0.5 1 1">
                <circle r={0.5} fill={dotColor}></circle>
            </svg>
            <div className="my-auto w-full text-center">{text}</div>
        </div>
    );
}
