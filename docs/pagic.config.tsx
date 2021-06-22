// @ts-ignore
import { React } from 'https://deno.land/x/pagic@v1.3.1/mod.ts';

export default {
    title: "Dorea DB Docs",
    theme: "docs",
    root: "/docs/",
    plugins: [
        "i18n",
        "sidebar"
    ],
    nav: [
        {
            text: '关于',
            link: 'https://blog.wwsg18.com/index.php/about.html',
            align: 'right',
        },
    ],
    sidebar: {
        "/": [
            "README.md",
            {
                link: "getting_started.md",
                children: [
                    "getting_started/installation.md"
                ]
            }
        ],

    },
    github: "https://github.com/mrxiaozhuox/Dorea"
};
