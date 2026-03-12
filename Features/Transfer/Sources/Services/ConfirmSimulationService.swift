// Copyright (c). Gem Wallet. All rights reserved.

import AddressNameService
import AssetsService
import Primitives
import PrimitivesComponents

public struct ConfirmSimulationService: Sendable {
    private let addressNameService: AddressNameService
    private let assetsService: AssetsService

    public init(
        addressNameService: AddressNameService,
        assetsService: AssetsService
    ) {
        self.addressNameService = addressNameService
        self.assetsService = assetsService
    }

    func makeState(data: TransferData, simulation: SimulationResult?) -> ConfirmSimulationState {
        return buildState(
            simulation: simulation,
            payload: payloadFields(for: data.type, simulation: simulation),
            payloadAddressNames: [:],
            headerData: cachedHeaderData(data: data, simulation: simulation)
        )
    }

    func updateState(data: TransferData, simulation: SimulationResult?) async -> ConfirmSimulationState {
        let payload = payloadFields(for: data.type, simulation: simulation)
        async let names = payloadAddressNames(chain: data.chain, payload: payload)
        async let headerData = headerData(data: data, simulation: simulation)

        return buildState(
            simulation: simulation,
            payload: payload,
            payloadAddressNames: await names,
            headerData: await headerData
        )
    }
}

private extension ConfirmSimulationService {
    func buildState(
        simulation: SimulationResult?,
        payload: [SimulationPayloadField],
        payloadAddressNames: [ChainAddress: AddressName],
        headerData: AssetValueHeaderData?
    ) -> ConfirmSimulationState {
        ConfirmSimulationState(
            warnings: simulation?.warnings ?? [],
            primaryFields: payload.filter { $0.display == .primary },
            secondaryFields: payload.filter { $0.display == .secondary },
            payloadAddressNames: payloadAddressNames,
            headerData: headerData
        )
    }

    func payloadFields(
        for transferType: TransferDataType,
        simulation: SimulationResult?
    ) -> [SimulationPayloadField] {
        guard case .generic = transferType else {
            return []
        }

        let payload = simulation?.payload ?? []
        guard shouldHideValueField(for: transferType, simulation: simulation) else {
            return payload
        }

        return payload.filter { $0.kind != .value }
    }

    func approvalHeaderData(for transferType: TransferDataType) -> AssetValueHeaderData? {
        guard case .tokenApprove(let asset, let approval) = transferType,
              let value = ApprovalValue(rawValue: approval.value) else {
            return nil
        }

        return AssetValueHeaderData(asset: asset, value: value)
    }

    func cachedHeaderData(data: TransferData, simulation: SimulationResult?) -> AssetValueHeaderData? {
        if let headerData = approvalHeaderData(for: data.type) {
            return headerData
        }

        guard case .generic = data.type,
              let headerValue = simulationHeaderValue(simulation) else {
            return nil
        }

        do {
            if let asset = try assetsService.getAssets(for: [headerValue.assetId]).first {
                return AssetValueHeaderData(asset: asset, value: headerValue.value)
            }
        } catch {
            return nil
        }
        
        return nil
    }

    func headerData(data: TransferData, simulation: SimulationResult?) async -> AssetValueHeaderData? {
        if let headerData = cachedHeaderData(data: data, simulation: simulation) {
            return headerData
        }

        guard case .generic = data.type,
              let headerValue = simulationHeaderValue(simulation) else {
            return nil
        }

        do {
            let asset = try await assetsService.getOrFetchTokenAsset(for: headerValue.assetId)
            return AssetValueHeaderData(asset: asset, value: headerValue.value)
        } catch {
            if !error.isCancelled {
                debugLog("simulation header asset error: \(error)")
            }
            return nil
        }
    }

    func payloadAddressNames(chain: Chain, payload: [SimulationPayloadField]) async -> [ChainAddress: AddressName] {
        let requests = payloadAddressRequests(chain: chain, payload: payload)

        do {
            return try await addressNameService.getAddressNames(requests: requests)
        } catch {
            if !error.isCancelled {
                debugLog("payload address name lookup error: \(error)")
            }
            return [:]
        }
    }

    func payloadAddressRequests(chain: Chain, payload: [SimulationPayloadField]) -> [ChainAddress] {
        payload.compactMap {
            guard $0.fieldType == .address else {
                return nil
            }
            return ChainAddress(chain: chain, address: $0.value)
        }
    }

    func shouldHideValueField(for transferType: TransferDataType, simulation: SimulationResult?) -> Bool {
        if approvalHeaderData(for: transferType) != nil {
            return true
        }

        return simulationHeaderValue(simulation) != nil
    }

    func simulationHeaderValue(_ simulation: SimulationResult?) -> (assetId: AssetId, value: ApprovalValue)? {
        guard let header = simulation?.header,
              let value = ApprovalValue(rawValue: header.value) else {
            return nil
        }
        return (header.assetId, value)
    }
}
