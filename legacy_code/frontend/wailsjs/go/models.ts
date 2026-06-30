export namespace main {
	
	export class ProgramaInfo {
	    nome: string;
	    caminho: string;
	
	    static createFrom(source: any = {}) {
	        return new ProgramaInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.nome = source["nome"];
	        this.caminho = source["caminho"];
	    }
	}
	export class TecladoInfo {
	    id: string;
	    nome: string;
	    tagIdioma: string;
	
	    static createFrom(source: any = {}) {
	        return new TecladoInfo(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.nome = source["nome"];
	        this.tagIdioma = source["tagIdioma"];
	    }
	}

}

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
	    mascaraRede: string;
	    gatewayPadrao: string;
	    dnsPrimario: string;
	    dnsSecundario: string;
	    interfaceAtiva: string;
	
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
	        this.mascaraRede = source["mascaraRede"];
	        this.gatewayPadrao = source["gatewayPadrao"];
	        this.dnsPrimario = source["dnsPrimario"];
	        this.dnsSecundario = source["dnsSecundario"];
	        this.interfaceAtiva = source["interfaceAtiva"];
	    }
	}

}

