import {All, Controller, Get, MethodNotAllowedException, Res} from "@nestjs/common";
import type {Response} from "express";
import {encode} from "@toon-format/toon";
const version = "1.0.0";

@Controller()
export class RootController {
    @Get()
    public async index(@Res() response: Response): Promise<void> {
        response
            .status(200)
            .type("text/plain; charset=utf-8")
            .send(encode({message: "Stock Prices API", version}));
    }

    @All()
    public methodNotAllowed(): never {
        throw new MethodNotAllowedException();
    }
}
