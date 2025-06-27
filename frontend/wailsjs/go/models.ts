export namespace sysinfo {
	
	export class InfoSistema {
	    nomeComputador: string;
	    versaoWindows: string;
	    edicaoWindows: string;
	    buildWindows: string;
	    processador: string;
	    memoriaTotalGB: string;
	    enderecoMAC: string;
	    enderecoIP: string;
	
	    static createFrom(source: any = {}) {
	        return new InfoSistema(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.nomeComputador = source["nomeComputador"];
	        this.versaoWindows = source["versaoWindows"];
	        this.edicaoWindows = source["edicaoWindows"];
	        this.buildWindows = source["buildWindows"];
	        this.processador = source["processador"];
	        this.memoriaTotalGB = source["memoriaTotalGB"];
	        this.enderecoMAC = source["enderecoMAC"];
	        this.enderecoIP = source["enderecoIP"];
	    }
	}

}

