// @ts-ignore
import { React } from 'https://deno.land/x/pagic@v1.3.1/mod.ts';

export default {
    title: "Dorea DB Docs",
    theme: "docs",
    plugins: [
        "i18n",
        "sidebar",
        "prev_next"
    ],
    nav: [
        {
            text: 'author',
            link: 'https://blog.wwsg18.com/index.php/about.html',
            align: 'right',
        },
    ],
    sidebar: {
        "/": [
            "README.md",
            {
                link: "installation.md",
            },
            {
                link: "connection.md",
            },
            {
                link: "client.md"
            }
        ],

    },
    github: "https://github.com/mrxiaozhuox/Dorea"
};
