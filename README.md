Die benutzen Branches von iced und iced_aw sind nicht stable, es kann also sein dass der Code in einigen Wochen so nicht mehr funktioniert.
Sollte das der Fall sein, wenn Sie den Code ausführen wollen, müssten Sie iced_aw mit der hier benutzen Revision kopieren und lokal darauf verweisen.
In dessen Cargo.toml sollten Sie dann unter [dependencies.iced] rev = "acf6daff46f8a0363a46f1957f481fa6625c7790" spezifizieren und in der Cargo.toml zu diesem Projekt dasselbe tun.
Tut mir leid, ich weiß leider nicht, wie das besser geht, ohne die veränderte Version von iced_aw hier mit hochzuladen.
