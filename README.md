Die benutzen Branches von iced und iced_aw sind nicht stable, es kann also sein dass der Code in einigen Wochen so nicht mehr funktioniert.
Sollte das der Fall sein, wenn Sie den Code ausführen wollen, müssten Sie iced_aw mit der hier benutzen Revision kopieren und lokal darauf verweisen.
In dessen Cargo.toml sollten Sie dann unter [dependencies.iced] rev = "950bfc07d4b71016bf3e9d53709395e185420cec" spezifizieren und in der Cargo.toml zu diesem Projekt dasselbe tun.
Tut mir leid, ich weiß leider nicht, wie das besser geht, ohne die veränderte Version von iced_aw hier mit hochzuladen.
