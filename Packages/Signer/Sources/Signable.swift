import Foundation
import Primitives

public enum SignMessage {
    case typed(String)
    case raw(Data)
}

public protocol Signable {
    func signData(input: SignerInput,privateKey: Data) throws -> String
    func signTransfer(input: SignerInput, privateKey: Data) throws -> String
    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String
    func signNftTransfer(input: SignerInput, privateKey: Data) throws -> String
    func signSwap(input: SignerInput, privateKey: Data) throws -> [String]
    func signTokenApproval(input: SignerInput, privateKey: Data) throws -> String
    func signStake(input: SignerInput, privateKey: Data) throws -> [String]
    func signEarn(input: SignerInput, privateKey: Data) throws -> [String]
    func signMessage(message: SignMessage, privateKey: Data) throws -> String
    func signAccountAction(input: SignerInput, privateKey: Data) throws -> String
    func signPerpetual(input: SignerInput, privateKey: Data) throws -> [String]
    func signWithdrawal(input: SignerInput, privateKey: Data) throws -> String
}

extension Signable {
    public func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signTokenTransfer method")
    }
    
    public func signNftTransfer(input: SignerInput, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signNftTransfer method")
    }
    
    public func signAccountAction(input: SignerInput, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signAccountAction method")
    }
    
    public func signData(input: Primitives.SignerInput, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signData method")
    }
    
    public func signSwap(input: SignerInput, privateKey: Data) throws -> [String] {
        throw AnyError("unimplemented: signSwap method")
    }
    
    public func signTokenApproval(input: SignerInput, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signTokenApproval method")
    }
    
    public func signStake(input: SignerInput, privateKey: Data) throws -> [String] {
        throw AnyError("unimplemented: signStake method")
    }

    public func signEarn(input: SignerInput, privateKey: Data) throws -> [String] {
        throw AnyError("unimplemented: signEarn method")
    }

    public func signMessage(message: SignMessage, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signMessage method")
    }
    
    public func signPerpetual(input: SignerInput, privateKey: Data) throws -> [String] {
        throw AnyError("unimplemented: signPerpetual method")
    }
    
    public func signWithdrawal(input: SignerInput, privateKey: Data) throws -> String {
        throw AnyError("unimplemented: signWithdrawal method")
    }
}
