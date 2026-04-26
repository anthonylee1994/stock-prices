import type {Response} from "express";
import {encode} from "@toon-format/toon";
import {Controller, Get, Res} from "@nestjs/common";

@Controller()
export class AppController {
    @Get()
    getRoot(@Res() res: Response): void {
        res.send(encode({message: "Stock Prices API", version: "1.0.0"}));
    }
}
