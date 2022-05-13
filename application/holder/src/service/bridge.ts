import {
    SetupService,
    Interface,
    Implementation,
    register,
    DependOn,
} from '@platform/entity/service';
import { services } from '@register/services';
import { electron } from '@service/electron';
import { Subscriber } from '@platform/env/subscription';

import * as Requests from '@platform/ipc/request';
import * as RequestHandlers from './bridge/index';

@DependOn(electron)
@SetupService(services['bridge'])
export class Service extends Implementation {
    private _subscriber: Subscriber = new Subscriber();
    public override ready(): Promise<void> {
        this._subscriber.register(
            electron
                .ipc()
                .respondent(
                    this.getName(),
                    Requests.File.Open.Request,
                    RequestHandlers.File.Open.handler,
                ),
        );
        this._subscriber.register(
            electron
                .ipc()
                .respondent(
                    this.getName(),
                    Requests.Connect.Dlt.Request,
                    RequestHandlers.Connect.Dlt.handler,
                ),
        );
        this._subscriber.register(
            electron
                .ipc()
                .respondent(
                    this.getName(),
                    Requests.File.Select.Request,
                    RequestHandlers.File.Select.handler,
                ),
        );
        this._subscriber.register(
            electron
                .ipc()
                .respondent(
                    this.getName(),
                    Requests.Dlt.Stat.Request,
                    RequestHandlers.Dlt.Stat.handler,
                ),
        );
        return Promise.resolve();
    }
}
export interface Service extends Interface {}
export const bridge = register(new Service());